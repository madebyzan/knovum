from __future__ import annotations

from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import Any

from pydantic import BaseModel

from knovum_inference.contracts.generated.alignment import SpeechAlignment, SpeechAlignParams
from knovum_inference.contracts.generated.control import RuntimeCapability
from knovum_inference.contracts.generated.diarization import SpeechDiarization, SpeechDiarizeParams
from knovum_inference.contracts.generated.embedding import EmbeddingEncodeParams, EmbeddingEncoding
from knovum_inference.contracts.generated.image import ImageSegmentation, ImageSegmentParams
from knovum_inference.contracts.generated.separation import SpeechSeparateParams, SpeechSeparation
from knovum_inference.contracts.generated.synthesis import SpeechSynthesis, SpeechSynthesizeParams
from knovum_inference.contracts.generated.transcription import SpeechTranscribeParams, SpeechTranscription
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.runtime import execution
from knovum_inference.runtime.active_request import ActiveRequest
from knovum_inference.runtime.session import Session
from knovum_inference.transport.protocol import ProtocolWriter
from knovum_inference.transport.validation import validate_params

InferenceParams = (
    EmbeddingEncodeParams
    | ImageSegmentParams
    | SpeechAlignParams
    | SpeechDiarizeParams
    | SpeechSeparateParams
    | SpeechSynthesizeParams
    | SpeechTranscribeParams
)


@dataclass(frozen=True, slots=True)
class InferenceMethod:
    capability: RuntimeCapability
    params_model: type[InferenceParams]
    response_model: type[BaseModel]
    execute: Callable[[Any, Session, ActiveRequest, ProtocolWriter], Awaitable[dict[str, Any]]]

    def decode(self, raw: dict[str, Any]) -> InferenceParams:
        params = validate_params(self.params_model, raw)
        if isinstance(params, EmbeddingEncodeParams) and (
            not 1 <= len(params.texts) <= 32 or any(not text for text in params.texts)
        ):
            raise invalid_params("Embedding requires 1 to 32 non-empty texts")
        return params


INFERENCE_METHODS = {
    "embedding.encode": InferenceMethod(
        RuntimeCapability.embedding, EmbeddingEncodeParams, EmbeddingEncoding, execution.embedding
    ),
    "speech.transcribe": InferenceMethod(
        RuntimeCapability.speech_transcription,
        SpeechTranscribeParams,
        SpeechTranscription,
        execution.speech_transcribe,
    ),
    "speech.align": InferenceMethod(
        RuntimeCapability.speech_alignment, SpeechAlignParams, SpeechAlignment, execution.speech_align
    ),
    "speech.diarize": InferenceMethod(
        RuntimeCapability.speech_diarization,
        SpeechDiarizeParams,
        SpeechDiarization,
        execution.speech_diarize,
    ),
    "speech.synthesize": InferenceMethod(
        RuntimeCapability.speech_synthesis,
        SpeechSynthesizeParams,
        SpeechSynthesis,
        execution.speech_synthesize,
    ),
    "speech.separate": InferenceMethod(
        RuntimeCapability.speech_separation,
        SpeechSeparateParams,
        SpeechSeparation,
        execution.speech_separate,
    ),
    "image.segment": InferenceMethod(
        RuntimeCapability.image_segmentation, ImageSegmentParams, ImageSegmentation, execution.image_segment
    ),
}


def capability_method(capability: RuntimeCapability) -> str:
    for name, method in INFERENCE_METHODS.items():
        if method.capability is capability:
            return name
    raise server_error("runtime_mismatch", "Runtime capability is not implemented")
