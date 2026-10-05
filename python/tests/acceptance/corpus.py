"""Fixed corpus definitions and qualification coverage requirements."""

from __future__ import annotations

from collections.abc import Sequence
from typing import Annotated, Literal, Self

from pydantic import Field, model_validator

from acceptance.contracts import (
    EvidenceModel,
    HashedFile,
    Language,
    Nonnegative,
    Positive,
    SchemaVersion,
    Text,
    VoiceScores,
)

ASR_COVERAGE_TAGS = {
    "mixed_language",
    "proper_noun",
    "numbers",
    "silence",
    "noise",
    "speaker_overlap",
    "long_recording",
}


class ReferenceProfile(EvidenceModel):
    model_spec_id: Text
    config_id: Text
    device: Literal["cpu"]
    precision: Literal["float32"]


class RetrievalDocument(EvidenceModel):
    id: Text
    kind: Literal["document", "note", "code", "transcript"]
    text: Text


class RetrievalQuery(EvidenceModel):
    id: Text
    text: Text
    relevant_document_ids: Annotated[list[Text], Field(min_length=1)]


class EmbeddingCorpus(EvidenceModel):
    reference_profile: ReferenceProfile
    documents: Annotated[list[RetrievalDocument], Field(min_length=1)]
    queries: Annotated[list[RetrievalQuery], Field(min_length=1)]

    @model_validator(mode="after")
    def validate_retrieval(self) -> Self:
        document_ids = {item.id for item in self.documents}
        if len(document_ids) != len(self.documents) or len({item.id for item in self.queries}) != len(self.queries):
            raise ValueError("embedding corpus identifiers must be unique")
        if {item.kind for item in self.documents} != {"document", "note", "code", "transcript"}:
            raise ValueError("embedding corpus must cover document, note, code, and transcript retrieval")
        if any(not set(item.relevant_document_ids).issubset(document_ids) for item in self.queries):
            raise ValueError("query relevance must identify existing documents")
        return self


class BaselineProfile(EvidenceModel):
    model_spec_id: Text
    config_id: Text
    model_family: Text
    model_size: Text


class AudioSample(EvidenceModel):
    id: Text
    language: Language
    audio: HashedFile
    duration_ms: Positive
    sample_rate_hz: Annotated[int, Field(gt=0)]


class TranscriptionSample(AudioSample):
    coverage_tags: list[Text]
    reference_text: str


class TranscriptionCorpus(EvidenceModel):
    baseline_profile: BaselineProfile
    samples: list[TranscriptionSample]

    @model_validator(mode="after")
    def validate_coverage(self) -> Self:
        validate_bilingual_samples(self.samples)
        tags = {tag for item in self.samples for tag in item.coverage_tags}
        if (
            not ASR_COVERAGE_TAGS.issubset(tags)
            or len({item.sample_rate_hz for item in self.samples}) < 2
            or not any(item.duration_ms >= 600_000 for item in self.samples)
        ):
            raise ValueError("ASR corpus requires all content conditions, two sample rates, and a ten-minute recording")
        return self


class AnnotatedUnit(EvidenceModel):
    id: Text
    text: str
    start_ms: Nonnegative
    end_ms: Nonnegative

    @model_validator(mode="after")
    def validate_boundaries(self) -> Self:
        if self.end_ms < self.start_ms:
            raise ValueError("annotation finishes before it starts")
        return self


class AlignmentSample(AudioSample):
    units: Annotated[list[AnnotatedUnit], Field(min_length=1)]

    @model_validator(mode="after")
    def validate_identities(self) -> Self:
        if len({unit.id for unit in self.units}) != len(self.units):
            raise ValueError("alignment annotation identifiers must be unique")
        return self


class AlignmentThresholds(EvidenceModel):
    minimum_coverage: Annotated[float, Field(ge=0, le=1)]
    maximum_median_boundary_error_ms: Nonnegative
    maximum_p95_boundary_error_ms: Nonnegative

    @model_validator(mode="after")
    def validate_limits(self) -> Self:
        if self.maximum_p95_boundary_error_ms < self.maximum_median_boundary_error_ms:
            raise ValueError("alignment thresholds are inconsistent")
        return self


class AlignmentCorpus(EvidenceModel):
    thresholds: AlignmentThresholds
    samples: list[AlignmentSample]

    @model_validator(mode="after")
    def validate_coverage(self) -> Self:
        validate_bilingual_samples(self.samples)
        return self


class SynthesisSample(EvidenceModel):
    id: Text
    language: Language
    text: Text


class SynthesisCorpus(EvidenceModel):
    thresholds: VoiceScores
    samples: Annotated[list[SynthesisSample], Field(min_length=30)]

    @model_validator(mode="after")
    def validate_coverage(self) -> Self:
        validate_bilingual_samples(self.samples)
        return self


def validate_bilingual_samples(samples: Sequence[AudioSample | SynthesisSample]) -> None:
    if len({item.id for item in samples}) != len(samples):
        raise ValueError("corpus sample identifiers must be unique")
    if {item.language for item in samples} != {"zh-CN", "en-US"}:
        raise ValueError("corpus samples must cover exactly zh-CN and en-US")


class Corpus(EvidenceModel):
    schema_version: SchemaVersion
    corpus_id: Text
    corpus_version: Text
    description: Text
    embedding: EmbeddingCorpus
    asr: TranscriptionCorpus
    alignment: AlignmentCorpus
    tts: SynthesisCorpus
