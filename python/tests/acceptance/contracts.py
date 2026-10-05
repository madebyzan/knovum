"""Strict current-version contracts for external qualification evidence."""

from __future__ import annotations

from typing import Annotated, Literal

from pydantic import BaseModel, ConfigDict, Field
from pydantic.alias_generators import to_camel

Text = Annotated[str, Field(min_length=1, pattern=r"\S")]
Sha256 = Annotated[str, Field(pattern=r"^[0-9a-f]{64}$")]
SchemaVersion = Annotated[int, Field(ge=1, le=1)]
Language = Literal["zh-CN", "en-US"]
Capability = Literal["embedding", "asr", "alignment", "tts"]
Nonnegative = Annotated[float, Field(ge=0)]
Positive = Annotated[float, Field(gt=0)]
Rating = Annotated[float, Field(ge=1, le=5)]


class EvidenceModel(BaseModel):
    model_config = ConfigDict(
        strict=True,
        extra="forbid",
        allow_inf_nan=False,
        alias_generator=to_camel,
        validate_by_alias=True,
        validate_by_name=False,
    )


class HashedFile(EvidenceModel):
    path: Text
    sha256: Sha256


class VoiceScores(EvidenceModel):
    pronunciation: Rating
    naturalness: Rating
    voice_consistency: Rating
    reference_similarity: Rating
    emotion_control: Rating
