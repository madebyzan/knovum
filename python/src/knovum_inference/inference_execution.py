from __future__ import annotations

import asyncio
from typing import Any

from knovum_inference.active_request import ActiveRequest
from knovum_inference.contracts import (
    EmbeddingEncodeParams,
    ImageSegmentParams,
    SpeechAlignParams,
    SpeechDiarizeParams,
    SpeechSeparateParams,
    SpeechSynthesizeParams,
    SpeechTranscribeParams,
    TensorDataType,
)
from knovum_inference.embedding import EXPECTED_DIMENSIONS
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.protocol import ProtocolWriter
from knovum_inference.runtime_initialization import Session
from knovum_inference.speech_stream import ChunkCredits, SynthesisChunkPublisher


async def embedding(
    params: EmbeddingEncodeParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    if params.embedding_spec_id != session.capability_spec_id:
        raise server_error("runtime_mismatch", "Embedding specification does not match")
    vectors, input_tokens = await asyncio.to_thread(session.engine.encode, params.texts, params.role.value)
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

    value: dict[str, Any] = await asyncio.to_thread(session.engine.transcribe, params, progress, active.cancelled)
    return value


async def speech_align(
    params: SpeechAlignParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    value: dict[str, Any] = await asyncio.to_thread(session.engine.align, params)
    return value


async def speech_diarize(
    params: SpeechDiarizeParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    value: dict[str, Any] = await asyncio.to_thread(session.engine.diarize, params)
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
    value: dict[str, Any] = await asyncio.to_thread(session.engine.synthesize, params, publisher, active.cancelled)
    return value


async def speech_separate(
    params: SpeechSeparateParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    value: dict[str, Any] = await asyncio.to_thread(
        session.engine.separate,
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
    mask = await asyncio.to_thread(
        session.engine.segment,
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
