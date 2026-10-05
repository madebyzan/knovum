from __future__ import annotations

from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import Any

from pydantic import BaseModel

from knovum_inference import inference_execution
from knovum_inference.active_request import ActiveRequest
from knovum_inference.contracts import (
    EmbeddingEncodeParams,
    EmbeddingEncoding,
    ImageSegmentation,
    ImageSegmentParams,
    RuntimeCapability,
    SpeechAlignment,
    SpeechAlignParams,
    SpeechDiarization,
    SpeechDiarizeParams,
    SpeechSeparateParams,
    SpeechSeparation,
    SpeechSynthesis,
    SpeechSynthesizeParams,
    SpeechTranscribeParams,
    SpeechTranscription,
)
from knovum_inference.errors import invalid_params
from knovum_inference.protocol import ProtocolWriter
from knovum_inference.request_contract import validate_params
from knovum_inference.runtime_initialization import Session


@dataclass(frozen=True, slots=True)
class InferenceMethod:
    capability: RuntimeCapability
    params_model: type[BaseModel]
    response_model: type[BaseModel]
    execute: Callable[[Any, Session, ActiveRequest, ProtocolWriter], Awaitable[dict[str, Any]]]

    def decode(self, raw: dict[str, Any]) -> BaseModel:
        params = validate_params(self.params_model, raw)
        if isinstance(params, EmbeddingEncodeParams) and (
            not 1 <= len(params.texts) <= 32 or any(not text for text in params.texts)
        ):
            raise invalid_params("Embedding requires 1 to 32 non-empty texts")
        return params


INFERENCE_METHODS = {
    "embedding.encode": InferenceMethod(
        RuntimeCapability.embedding, EmbeddingEncodeParams, EmbeddingEncoding, inference_execution.embedding
    ),
    "speech.transcribe": InferenceMethod(
        RuntimeCapability.speech_transcription,
        SpeechTranscribeParams,
        SpeechTranscription,
        inference_execution.speech_transcribe,
    ),
    "speech.align": InferenceMethod(
        RuntimeCapability.speech_alignment, SpeechAlignParams, SpeechAlignment, inference_execution.speech_align
    ),
    "speech.diarize": InferenceMethod(
        RuntimeCapability.speech_diarization,
        SpeechDiarizeParams,
        SpeechDiarization,
        inference_execution.speech_diarize,
    ),
    "speech.synthesize": InferenceMethod(
        RuntimeCapability.speech_synthesis,
        SpeechSynthesizeParams,
        SpeechSynthesis,
        inference_execution.speech_synthesize,
    ),
    "speech.separate": InferenceMethod(
        RuntimeCapability.speech_separation,
        SpeechSeparateParams,
        SpeechSeparation,
        inference_execution.speech_separate,
    ),
    "image.segment": InferenceMethod(
        RuntimeCapability.image_segmentation, ImageSegmentParams, ImageSegmentation, inference_execution.image_segment
    ),
}
