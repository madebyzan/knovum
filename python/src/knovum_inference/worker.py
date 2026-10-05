from __future__ import annotations

import asyncio
import json
import os
import signal
import sys
import threading
import time
from typing import Any

from knovum_inference import PROTOCOL_VERSION
from knovum_inference.active_request import ActiveRequest
from knovum_inference.contracts import (
    RuntimeAckChunkParams,
    RuntimeCancelParams,
    RuntimeCancelResult,
    RuntimeCapability,
    RuntimeHealthParams,
    RuntimeHealthResult,
    RuntimeInitializeParams,
    RuntimeInitializeResult,
    RuntimeShutdownParams,
    RuntimeShutdownResult,
)
from knovum_inference.errors import RpcError, invalid_params, server_error
from knovum_inference.inference_methods import INFERENCE_METHODS
from knovum_inference.input_stream import OVERSIZED, InputStream
from knovum_inference.protocol import MAX_MESSAGE_BYTES, ProtocolWriter, Request, parse_request
from knovum_inference.request_contract import report_request_error, validate_params, validate_result
from knovum_inference.runtime_health import current_process_resident_bytes
from knovum_inference.runtime_initialization import Session, capability_method, prepare_session

U64_MAX = (1 << 64) - 1


class Worker:
    def __init__(self) -> None:
        self._writer = ProtocolWriter()
        self._session: Session | None = None
        self._active: ActiveRequest | None = None
        self._accepting_work = True
        self._initializing = False
        self._terminate = asyncio.Event()
        self._tasks: set[asyncio.Task[object]] = set()

    async def serve(self) -> None:
        input_stream = InputStream(asyncio.get_running_loop())
        input_stream.start()
        try:
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
        finally:
            await self._settle_requests()

    async def _settle_requests(self) -> None:
        self._accepting_work = False
        self._terminate.set()
        if self._active is not None:
            self._active.cancelled.set()
        if self._tasks:
            try:
                await asyncio.wait_for(asyncio.gather(*self._tasks, return_exceptions=True), timeout=5)
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
        except Exception as error:
            if not isinstance(error, RpcError):
                print(
                    json.dumps({"event": "requestFailed", "method": request.method}),
                    file=sys.stderr,
                    flush=True,
                )
            if request.request_id is not None:
                await self._writer.error(request.request_id, report_request_error(error))

    async def _dispatch(self, request: Request) -> dict[str, Any]:
        if request.method == "runtime.initialize":
            return await self._initialize(request.params)
        if self._session is None:
            raise server_error("runtime_mismatch", "Runtime is not initialized")
        handlers = {
            "runtime.health": self._health,
            "runtime.cancel": self._cancel,
            "runtime.ackChunk": self._ack_chunk,
            "runtime.shutdown": self._shutdown,
        }
        method = INFERENCE_METHODS.get(request.method)
        if method is not None:
            params = method.decode(request.params)
            session, active = self._begin(params, method.capability)
            try:
                value = await method.execute(params, session, active, self._writer)
                self._check_active(params, active)
                return validate_result(method.result_model, value)
            finally:
                self._complete(active)
        handler = handlers.get(request.method)
        if handler is None:
            raise RpcError(-32601, "Method not found", "invalid_output")
        return await handler(request.params)

    async def _initialize(self, raw: dict[str, Any]) -> dict[str, Any]:
        if self._session is not None or self._initializing:
            raise server_error("runtime_mismatch", "Runtime is already initialized or initializing")
        if not self._accepting_work:
            raise server_error("cancelled", "Runtime is stopping")
        params = validate_params(RuntimeInitializeParams, raw)
        self._initializing = True
        try:
            session = await asyncio.to_thread(prepare_session, params)
            if not self._accepting_work:
                raise server_error("cancelled", "Runtime stopped during initialization")
            self._session = session
        finally:
            self._initializing = False
        return validate_result(
            RuntimeInitializeResult,
            {
                "protocolVersion": PROTOCOL_VERSION,
                "runtimeDigest": params.runtime_digest,
                "sessionId": params.session_id,
                "capability": params.capability.value,
                "capabilitySpecId": params.capability_spec_id,
                "deviceProfile": params.device_profile.model_dump(by_alias=True),
                "capabilities": [capability_method(params.capability)],
                "limits": {
                    "maxMessageBytes": MAX_MESSAGE_BYTES,
                    "maxEmbeddingTexts": 32,
                    "maxEmbeddingTokens": 2048,
                    "maxUnconsumedChunkFiles": 8,
                    "maxUnconsumedChunkBytes": 16 * 1024 * 1024,
                },
            },
        )

    async def _health(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = validate_params(RuntimeHealthParams, raw)
        self._require_session(params.session_id)
        return validate_result(
            RuntimeHealthResult,
            {
                "sessionId": params.session_id,
                "currentRequestId": self._active.request_id if self._active else None,
                "residentBytes": current_process_resident_bytes(),
            },
        )

    async def _cancel(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = validate_params(RuntimeCancelParams, raw)
        if params.attempt_epoch > U64_MAX:
            raise invalid_params("Attempt epoch exceeds uint64")
        active = self._active
        accepted = bool(
            active and active.request_id == params.target_request_id and active.attempt_epoch == params.attempt_epoch
        )
        if accepted and active is not None:
            active.cancelled.set()
        return validate_result(RuntimeCancelResult, {"accepted": accepted})

    async def _ack_chunk(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = validate_params(RuntimeAckChunkParams, raw)
        active = self._active
        if active is None or active.chunk_credits is None:
            raise server_error("invalid_output", "No synthesis stream is awaiting chunk credit")
        active.chunk_credits.acknowledge(
            params.request_id,
            params.attempt_epoch,
            params.seq,
            params.consumed_bytes,
        )
        return {}

    async def _shutdown(self, raw: dict[str, Any]) -> dict[str, Any]:
        params = validate_params(RuntimeShutdownParams, raw)
        self._require_session(params.session_id)
        self._accepting_work = False
        active = self._active
        if active is not None:
            active.cancelled.set()
            await active.done.wait()
        return validate_result(RuntimeShutdownResult, {"accepted": True})

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
