from __future__ import annotations

import asyncio
import json
import os
import signal
import sys
import threading
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, TypeVar

from pydantic import BaseModel, ValidationError

from knovum_inference import PROTOCOL_VERSION
from knovum_inference.contracts import (
    EmbeddingEncodeParams,
    EmbeddingEncodeResult,
    ImageSegmentParams,
    ImageSegmentResult,
    RuntimeCancelParams,
    RuntimeCancelResult,
    RuntimeCapability,
    RuntimeHealthParams,
    RuntimeHealthResult,
    RuntimeInitializeParams,
    RuntimeInitializeResult,
    RuntimeShutdownParams,
    RuntimeShutdownResult,
    SpeechAlignParams,
    SpeechAlignResult,
    SpeechDiarizeParams,
    SpeechDiarizeResult,
    SpeechSynthesizeParams,
    SpeechSynthesizeResult,
    SpeechTranscribeParams,
    SpeechTranscribeResult,
    TensorDataType,
)
from knovum_inference.embedding import EXPECTED_DIMENSIONS, EmbeddingEngine
from knovum_inference.errors import RpcError, invalid_params, server_error
from knovum_inference.input_stream import OVERSIZED, InputStream
from knovum_inference.protocol import MAX_MESSAGE_BYTES, ProtocolWriter, Request, parse_request
from knovum_inference.runtime_identity import verify_runtime_digest
from knovum_inference.speech import (
    AlignmentEngine,
    DiarizationEngine,
    SynthesisEngine,
    TranscriptionEngine,
)

T = TypeVar("T", bound=BaseModel)
U64_MAX = (1 << 64) - 1


@dataclass(slots=True)
class Session:
    session_id: str
    capability: RuntimeCapability
    capability_spec_id: str
    engine: Any
    work_directory: Path


@dataclass(slots=True)
class ActiveRequest:
    request_id: str
    attempt_epoch: int
    cancelled: threading.Event
    done: asyncio.Event


def _validated(model: type[T], value: Any) -> T:
    try:
        encoded = json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(",", ":"))
        return model.model_validate_json(encoded)
    except (TypeError, ValueError, ValidationError) as error:
        raise invalid_params("Parameters do not match the runtime contract") from error


def _result(model: type[T], value: dict[str, Any]) -> dict[str, Any]:
    try:
        return model.model_validate(value).model_dump(by_alias=True, mode="json")
    except ValidationError as error:
        raise server_error("invalid_output", "Worker result violates its contract") from error


class Worker:
    def __init__(self) -> None:
        self._writer = ProtocolWriter()
        self._session: Session | None = None
        self._active: ActiveRequest | None = None
        self._accepting_work = True
        self._terminate = asyncio.Event()
        self._tasks: set[asyncio.Task[object]] = set()

    async def serve(self) -> None:
        input_stream = InputStream(asyncio.get_running_loop())
        input_stream.start()
        while not self._terminate.is_set():
            receive = asyncio.create_task(input_stream.receive())
            stopping = asyncio.create_task(self._terminate.wait())
            done, pending = await asyncio.wait({receive, stopping}, return_when=asyncio.FIRST_COMPLETED)
            for task in pending:
                task.cancel()
            if stopping in done:
                break
            line = receive.result()
            if line is None:
                break
            if line is OVERSIZED:
                await self._writer.error(None, RpcError(-32600, "Message exceeds 1 MiB", "invalid_output"))
                continue
            assert isinstance(line, bytes)
            try:
                request = parse_request(line)
            except RpcError as error:
                await self._writer.error(None, error)
                continue
            task = asyncio.create_task(self._handle(request))
            self._tasks.add(task)
            task.add_done_callback(self._tasks.discard)
        if self._active is not None:
            self._active.cancelled.set()
            try:
                await asyncio.wait_for(self._active.done.wait(), timeout=5)
            except TimeoutError:
                if os.name == "posix":
                    os.killpg(os.getpgrp(), signal.SIGKILL)
                os._exit(1)

    async def _handle(self, request: Request) -> None:
        try:
            result = await self._dispatch(request)
            if request.request_id is None:
                return
            await self._writer.result(request.request_id, result)
            if request.method == "runtime.shutdown":
                self._terminate.set()
        except RpcError as error:
            if request.request_id is not None:
                await self._writer.error(request.request_id, error)
        except Exception:
            print(
                json.dumps({"event": "requestFailed", "method": request.method}),
                file=sys.stderr,
                flush=True,
            )
            if request.request_id is not None:
                await self._writer.error(
                    request.request_id,
                    server_error("invalid_output", "Worker request failed"),
                )

    async def _dispatch(self, request: Request) -> dict[str, Any]:
        if request.method == "runtime.initialize":
            return self._initialize(request.params)
        if self._session is None:
            raise server_error("runtime_mismatch", "Runtime is not initialized")
        handlers = {
            "runtime.health": self._health,
            "runtime.cancel": self._cancel,
            "runtime.shutdown": self._shutdown,
            "embedding.encode": self._embedding,
            "image.segment": self._image_segment,
            "speech.transcribe": self._speech_transcribe,
            "speech.align": self._speech_align,
            "speech.diarize": self._speech_diarize,
            "speech.synthesize": self._speech_synthesize,
        }
        handler = handlers.get(request.method)
        if handler is None:
            raise RpcError(-32601, "Method not found", "invalid_output")
        return await handler(request.params)

    def _initialize(self, raw: dict[str, Any]) -> dict[str, Any]:
        if self._session is not None:
            raise server_error("runtime_mismatch", "Runtime is already initialized")
        params = _validated(RuntimeInitializeParams, raw)
        if params.protocol_version != PROTOCOL_VERSION:
            raise server_error("runtime_mismatch", "Protocol version does not match")
        if not params.session_id or not params.capability_spec_id or not params.device_profile.execution_profile_id:
            raise invalid_params("Runtime identity fields must be non-empty")
        if params.device_profile.cpu_threads < 1:
            raise server_error("unsupported_device", "CPU thread budget must be positive")
        work_directory = Path(params.work_directory)
        if not work_directory.is_absolute() or not work_directory.is_dir():
            raise invalid_params("Work directory must be an existing absolute directory")
        verify_runtime_digest(params.runtime_digest)
        model_path = self._selected_model_path(params)
        engine = self._create_engine(params, model_path, work_directory)
        self._session = Session(
            params.session_id,
            params.capability,
            params.capability_spec_id,
            engine,
            work_directory.resolve(strict=True),
        )
        return _result(
            RuntimeInitializeResult,
            {
                "protocolVersion": PROTOCOL_VERSION,
                "runtimeDigest": params.runtime_digest,
                "sessionId": params.session_id,
                "capability": params.capability.value,
                "capabilitySpecId": params.capability_spec_id,
                "deviceProfile": params.device_profile.model_dump(by_alias=True),
                "capabilities": [self._method(params.capability)],
                "limits": {
                    "maxMessageBytes": MAX_MESSAGE_BYTES,
                    "maxEmbeddingTexts": 32,
                    "maxEmbeddingTokens": 2048,
                },
            },
        )

    @staticmethod
    def _method(capability: RuntimeCapability) -> str:
        methods = {
            RuntimeCapability.embedding: "embedding.encode",
            RuntimeCapability.speech_transcription: "speech.transcribe",
            RuntimeCapability.speech_alignment: "speech.align",
            RuntimeCapability.speech_diarization: "speech.diarize",
            RuntimeCapability.speech_synthesis: "speech.synthesize",
            RuntimeCapability.image_segmentation: "image.segment",
        }
        method = methods.get(capability)
        if method is None:
            raise server_error("runtime_mismatch", "Runtime capability is not implemented")
        return method

    @staticmethod
    def _selected_model_path(params: RuntimeInitializeParams) -> Path:
        fields = {
            RuntimeCapability.embedding: params.model_paths.embedding,
            RuntimeCapability.speech_transcription: params.model_paths.speech_transcription,
            RuntimeCapability.speech_alignment: params.model_paths.speech_alignment,
            RuntimeCapability.speech_diarization: params.model_paths.speech_diarization,
            RuntimeCapability.speech_synthesis: params.model_paths.speech_synthesis,
            RuntimeCapability.image_segmentation: params.model_paths.image_segmentation,
        }
        selected = fields[params.capability]
        if selected is None or sum(value is not None for value in fields.values()) != 1:
            raise invalid_params("Exactly one capability model path is required")
        model_path = Path(selected)
        if not model_path.is_absolute() or not model_path.exists():
            raise server_error("model_missing", "Capability model path is missing")
        return model_path.resolve(strict=True)

    @staticmethod
    def _create_engine(params: RuntimeInitializeParams, model_path: Path, work_directory: Path) -> Any:
        capability = params.capability
        device = params.device_profile.device
        threads = params.device_profile.cpu_threads
        if capability is RuntimeCapability.embedding:
            if device != "cpu" or not model_path.is_dir():
                raise server_error("unsupported_device", "Embedding profile is unsupported")
            return EmbeddingEngine(str(model_path), device, threads)
        if capability is RuntimeCapability.speech_transcription:
            if not model_path.is_dir():
                raise server_error("model_missing", "ASR model directory is missing")
            return TranscriptionEngine(model_path, work_directory, threads)
        if capability is RuntimeCapability.speech_alignment:
            if not model_path.is_dir():
                raise server_error("model_missing", "Alignment model directory is missing")
            return AlignmentEngine(model_path, work_directory, device, threads)
        if capability is RuntimeCapability.speech_diarization:
            if not model_path.is_dir():
                raise server_error("model_missing", "Diarization model directory is missing")
            return DiarizationEngine(model_path, device)
        if capability is RuntimeCapability.speech_synthesis:
            if device != "cpu" or not model_path.is_dir():
                raise server_error("unsupported_device", "Synthesis profile is unsupported")
            return SynthesisEngine(model_path, work_directory, threads)
        if capability is RuntimeCapability.image_segmentation:
            if device != "cpu" or not model_path.is_file():
                raise server_error("unsupported_device", "Image segmentation profile is unsupported")
            from knovum_inference.image_segmentation import ImageSegmentationEngine

            return ImageSegmentationEngine(str(model_path), str(work_directory), threads)
        raise server_error("runtime_mismatch", "Runtime capability is not implemented")

    async def _health(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = _validated(RuntimeHealthParams, raw)
        self._require_session(params.session_id)
        return _result(
            RuntimeHealthResult,
            {
                "sessionId": params.session_id,
                "currentRequestId": self._active.request_id if self._active else None,
                "residentBytes": None,
            },
        )

    async def _cancel(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = _validated(RuntimeCancelParams, raw)
        if params.attempt_epoch > U64_MAX:
            raise invalid_params("Attempt epoch exceeds uint64")
        active = self._active
        accepted = bool(
            active and active.request_id == params.target_request_id and active.attempt_epoch == params.attempt_epoch
        )
        if accepted and active is not None:
            active.cancelled.set()
        return _result(RuntimeCancelResult, {"accepted": accepted})

    async def _shutdown(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = _validated(RuntimeShutdownParams, raw)
        self._require_session(params.session_id)
        self._accepting_work = False
        active = self._active
        if active is not None:
            active.cancelled.set()
            await active.done.wait()
        return _result(RuntimeShutdownResult, {"accepted": True})

    async def _embedding(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = _validated(EmbeddingEncodeParams, raw)
        if not 1 <= len(params.texts) <= 32 or any(not text for text in params.texts):
            raise invalid_params("Embedding requires 1 to 32 non-empty texts")
        session, active = self._begin(params, RuntimeCapability.embedding)
        try:
            if params.embedding_spec_id != session.capability_spec_id:
                raise server_error("runtime_mismatch", "Embedding specification does not match")
            vectors, input_tokens = await asyncio.to_thread(session.engine.encode, params.texts, params.role.value)
            self._check_active(params, active)
            return _result(
                EmbeddingEncodeResult,
                {
                    "requestId": params.request_id,
                    "attemptEpoch": params.attempt_epoch,
                    "embeddingSpecId": params.embedding_spec_id,
                    "vectors": vectors,
                    "dimensions": EXPECTED_DIMENSIONS,
                    "inputTokens": input_tokens,
                },
            )
        finally:
            self._complete(active)

    async def _speech_transcribe(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = _validated(SpeechTranscribeParams, raw)
        session, active = self._begin(params, RuntimeCapability.speech_transcription)
        loop = asyncio.get_running_loop()
        event_seq = 0

        def progress(
            stage: str, percent: int, language: str | None = None, partial: dict[str, Any] | None = None
        ) -> None:
            nonlocal event_seq
            if active.cancelled.is_set():
                raise server_error("cancelled", "Transcription was cancelled")
            event_seq += 1
            event = {
                "requestId": params.request_id,
                "attemptEpoch": params.attempt_epoch,
                "seq": event_seq,
                "eventType": "partial_transcript" if partial is not None else "progress",
                "stage": stage,
                "progress": percent,
                "language": language,
                "partial": partial,
            }
            asyncio.run_coroutine_threadsafe(self._writer.notification("speech.progress", event), loop).result(
                timeout=5
            )

        try:
            value = await asyncio.to_thread(session.engine.transcribe, params, progress, active.cancelled)
            self._check_active(params, active)
            return _result(SpeechTranscribeResult, value)
        finally:
            self._complete(active)

    async def _speech_align(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = _validated(SpeechAlignParams, raw)
        session, active = self._begin(params, RuntimeCapability.speech_alignment)
        try:
            value = await asyncio.to_thread(session.engine.align, params)
            self._check_active(params, active)
            return _result(SpeechAlignResult, value)
        finally:
            self._complete(active)

    async def _speech_diarize(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = _validated(SpeechDiarizeParams, raw)
        session, active = self._begin(params, RuntimeCapability.speech_diarization)
        try:
            value = await asyncio.to_thread(session.engine.diarize, params)
            self._check_active(params, active)
            return _result(SpeechDiarizeResult, value)
        finally:
            self._complete(active)

    async def _speech_synthesize(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = _validated(SpeechSynthesizeParams, raw)
        session, active = self._begin(params, RuntimeCapability.speech_synthesis)
        try:
            value = await asyncio.to_thread(session.engine.synthesize, params)
            self._check_active(params, active)
            return _result(SpeechSynthesizeResult, value)
        finally:
            self._complete(active)

    async def _image_segment(self, raw: dict[str, Any]) -> dict[str, Any]:
        from knovum_inference.image_segmentation import INPUT_BYTES, INPUT_SHAPE

        params = _validated(ImageSegmentParams, raw)
        session, active = self._begin(params, RuntimeCapability.image_segmentation)
        shape = tuple(item.root for item in params.tensor_ref.shape)
        if (
            params.tensor_ref.byte_length != INPUT_BYTES
            or shape != INPUT_SHAPE
            or params.tensor_ref.data_type is not TensorDataType.float32_little_endian
        ):
            self._complete(active)
            raise invalid_params("Image tensor metadata is invalid")
        try:
            mask = await asyncio.to_thread(
                session.engine.segment,
                params.tensor_ref.file_name,
                params.tensor_ref.sha256,
                params.request_id,
                active.cancelled,
            )
            self._check_active(params, active)
            return _result(
                ImageSegmentResult,
                {
                    "requestId": params.request_id,
                    "attemptEpoch": params.attempt_epoch,
                    "modelSpecId": params.model_spec_id,
                    "maskRef": {
                        "fileName": mask.file_name,
                        "byteLength": mask.byte_length,
                        "sha256": mask.sha256,
                        "shape": list(mask.shape),
                        "dataType": mask.data_type,
                    },
                },
            )
        finally:
            self._complete(active)

    def _begin(self, params: Any, capability: RuntimeCapability) -> tuple[Session, ActiveRequest]:
        if params.attempt_epoch > U64_MAX or params.deadline_ms > U64_MAX:
            raise invalid_params("Request identity exceeds uint64")
        identity = (
            params.request_id,
            params.owner_kind,
            params.owner_id,
            params.attempt_id,
            params.input_revision,
            params.model_spec_id,
        )
        if any(not value for value in identity):
            raise invalid_params("Request identity fields must be non-empty")
        session = self._session
        if session is None:
            raise server_error("runtime_mismatch", "Runtime is not initialized")
        if session.capability is not capability or params.model_spec_id != session.capability_spec_id:
            raise server_error("runtime_mismatch", "Request capability identity does not match")
        if not self._accepting_work or self._active is not None:
            raise server_error("resource_busy", "Worker is not accepting inference")
        if int(time.time() * 1000) >= params.deadline_ms:
            raise server_error("deadline_exceeded", "Request deadline has elapsed")
        active = ActiveRequest(params.request_id, params.attempt_epoch, threading.Event(), asyncio.Event())
        self._active = active
        return session, active

    @staticmethod
    def _check_active(params: Any, active: ActiveRequest) -> None:
        if active.cancelled.is_set():
            raise server_error("cancelled", "Inference request was cancelled")
        if int(time.time() * 1000) >= params.deadline_ms:
            raise server_error("deadline_exceeded", "Request deadline has elapsed")

    def _complete(self, active: ActiveRequest) -> None:
        if self._active is active:
            self._active = None
        active.done.set()

    def _require_session(self, session_id: str) -> None:
        if self._session is None or self._session.session_id != session_id:
            raise server_error("runtime_mismatch", "Runtime session does not match")
