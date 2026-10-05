from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING, Any, cast

from knovum_inference.contracts.generated.alignment import SpeechAlignParams
from knovum_inference.contracts.generated.diarization import SpeechDiarizeParams
from knovum_inference.contracts.generated.embedding import EmbeddingEncodeParams
from knovum_inference.contracts.generated.image import ImageSegmentParams, TensorDataType
from knovum_inference.contracts.generated.separation import SpeechSeparateParams
from knovum_inference.contracts.generated.synthesis import SpeechSynthesizeParams
from knovum_inference.contracts.generated.transcription import SpeechTranscribeParams
from knovum_inference.embedding import EXPECTED_DIMENSIONS
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.runtime.active_request import ActiveRequest
from knovum_inference.runtime.session import Session
from knovum_inference.speech.synthesis.stream import ChunkCredits, SynthesisChunkPublisher
from knovum_inference.transport.protocol import ProtocolWriter

if TYPE_CHECKING:
    from knovum_inference.embedding import EmbeddingEngine
    from knovum_inference.image_segmentation import ImageSegmentationEngine
    from knovum_inference.speech.alignment import AlignmentEngine
    from knovum_inference.speech.diarization import DiarizationEngine
    from knovum_inference.speech.separation import SpleeterSourceSeparator
    from knovum_inference.speech.synthesis.engine import SynthesisEngine
    from knovum_inference.speech.transcription import TranscriptionEngine

# Worker admission checks the capability; session composition selects its concrete engine.


async def embedding(
    params: EmbeddingEncodeParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    if params.embedding_spec_id != session.capability_spec_id:
        raise server_error("runtime_mismatch", "Embedding specification does not match")
    engine = cast("EmbeddingEngine", session.engine)
    vectors, input_tokens = await asyncio.to_thread(engine.encode, params.texts, params.role.value)
    return {
        "requestId": params.request_id,
        "attemptEpoch": params.attempt_epoch,
        "embeddingSpecId": params.embedding_spec_id,
        "vectors": vectors,
        "dimensions": EXPECTED_DIMENSIONS,
        "inputTokens": input_tokens,
    }


async def speech_transcribe(
    params: SpeechTranscribeParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    loop = asyncio.get_running_loop()
    event_seq = 0

    def progress(stage: str, percent: int, language: str | None = None, partial: dict[str, Any] | None = None) -> None:
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
        asyncio.run_coroutine_threadsafe(writer.notification("speech.progress", event), loop).result(timeout=5)

    engine = cast("TranscriptionEngine", session.engine)
    value = await asyncio.to_thread(engine.transcribe, params, progress, active.cancelled)
    return value


async def speech_align(
    params: SpeechAlignParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    engine = cast("AlignmentEngine", session.engine)
    value = await asyncio.to_thread(engine.align, params)
    return value


async def speech_diarize(
    params: SpeechDiarizeParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    engine = cast("DiarizationEngine", session.engine)
    value = await asyncio.to_thread(engine.diarize, params)
    return value


async def speech_synthesize(
    params: SpeechSynthesizeParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    active.chunk_credits = ChunkCredits(
        params.request_id,
        params.attempt_epoch,
        params.deadline_ms,
        active.cancelled,
    )
    loop = asyncio.get_running_loop()

    def publish(event: dict[str, Any]) -> None:
        if active.cancelled.is_set():
            raise server_error("cancelled", "Synthesis was cancelled")
        asyncio.run_coroutine_threadsafe(writer.notification("speech.chunk", event), loop).result(timeout=5)

    publisher = SynthesisChunkPublisher(
        session.work_directory,
        params.request_id,
        params.attempt_epoch,
        params.deadline_ms,
        active.chunk_credits,
        publish,
    )
    engine = cast("SynthesisEngine", session.engine)
    value = await asyncio.to_thread(engine.synthesize, params, publisher, active.cancelled)
    return value


async def speech_separate(
    params: SpeechSeparateParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    engine = cast("SpleeterSourceSeparator", session.engine)
    value = await asyncio.to_thread(
        engine.separate,
        params,
        active.cancelled,
    )
    return value


async def image_segment(
    params: ImageSegmentParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    from knovum_inference.image_segmentation import INPUT_BYTES, INPUT_SHAPE

    shape = tuple(item.root for item in params.tensor_ref.shape)
    if (
        params.tensor_ref.byte_length != INPUT_BYTES
        or shape != INPUT_SHAPE
        or params.tensor_ref.data_type is not TensorDataType.float32_little_endian
    ):
        raise invalid_params("Image tensor metadata is invalid")
    engine = cast("ImageSegmentationEngine", session.engine)
    mask = await asyncio.to_thread(
        engine.segment,
        params.tensor_ref.file_name,
        params.tensor_ref.sha256,
        params.request_id,
        active.cancelled,
    )
    return {
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
    }
