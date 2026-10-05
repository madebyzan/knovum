from __future__ import annotations

from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import Any, TypeVar, cast

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
Params = TypeVar("Params", bound=InferenceParams)


@dataclass(frozen=True, slots=True)
class InferenceMethod:
    capability: RuntimeCapability
    response_model: type[BaseModel]
    decode: Callable[[dict[str, Any]], InferenceParams]
    execute: Callable[[InferenceParams, Session, ActiveRequest, ProtocolWriter], Awaitable[dict[str, Any]]]

    @classmethod
    def bind(
        cls,
        capability: RuntimeCapability,
        params_model: type[Params],
        response_model: type[BaseModel],
        execute: Callable[[Params, Session, ActiveRequest, ProtocolWriter], Awaitable[dict[str, Any]]],
    ) -> InferenceMethod:
        def decode(raw: dict[str, Any]) -> InferenceParams:
            params = validate_params(params_model, raw)
            if isinstance(params, EmbeddingEncodeParams) and (
                not 1 <= len(params.texts) <= 32 or any(not text for text in params.texts)
            ):
                raise invalid_params("Embedding requires 1 to 32 non-empty texts")
            return params

        async def invoke(
            params: InferenceParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
        ) -> dict[str, Any]:
            # decode validates this exact model before the Worker invokes the bound execution.
            return await execute(cast(Params, params), session, active, writer)

        return cls(capability, response_model, decode, invoke)


INFERENCE_METHODS = {
    "embedding.encode": InferenceMethod.bind(
        RuntimeCapability.embedding, EmbeddingEncodeParams, EmbeddingEncoding, execution.embedding
    ),
    "speech.transcribe": InferenceMethod.bind(
        RuntimeCapability.speech_transcription,
        SpeechTranscribeParams,
        SpeechTranscription,
        execution.speech_transcribe,
    ),
    "speech.align": InferenceMethod.bind(
        RuntimeCapability.speech_alignment, SpeechAlignParams, SpeechAlignment, execution.speech_align
    ),
    "speech.diarize": InferenceMethod.bind(
        RuntimeCapability.speech_diarization,
        SpeechDiarizeParams,
        SpeechDiarization,
        execution.speech_diarize,
    ),
    "speech.synthesize": InferenceMethod.bind(
        RuntimeCapability.speech_synthesis,
        SpeechSynthesizeParams,
        SpeechSynthesis,
        execution.speech_synthesize,
    ),
    "speech.separate": InferenceMethod.bind(
        RuntimeCapability.speech_separation,
        SpeechSeparateParams,
        SpeechSeparation,
        execution.speech_separate,
    ),
    "image.segment": InferenceMethod.bind(
        RuntimeCapability.image_segmentation, ImageSegmentParams, ImageSegmentation, execution.image_segment
    ),
}


def capability_method(capability: RuntimeCapability) -> str:
    for name, method in INFERENCE_METHODS.items():
        if method.capability is capability:
            return name
    raise server_error("runtime_mismatch", "Runtime capability is not implemented")
