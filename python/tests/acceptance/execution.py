"""Hash-bound worker/reference outputs and retained invocation identities."""

from __future__ import annotations

from typing import Annotated, Generic, Literal, Self, TypeVar

from pydantic import AwareDatetime, Field, model_validator

from acceptance.contracts import EvidenceModel, HashedFile, Nonnegative, SchemaVersion, Sha256, Text

Values = TypeVar("Values", bound=EvidenceModel)
SourceKind = Literal["bundled_worker", "official_python_reference", "product_baseline"]


class Invocation(EvidenceModel):
    request_id: Text
    attempt_epoch: Annotated[int, Field(ge=0)]
    started_at: AwareDatetime
    finished_at: AwareDatetime

    @model_validator(mode="after")
    def validate_timing(self) -> Self:
        if self.finished_at < self.started_at:
            raise ValueError("invocation finishes before it starts")
        return self


class ExecutionSource(EvidenceModel):
    kind: SourceKind
    method: Text
    runtime_digest: Sha256
    model_spec_id: Text
    config_id: Text
    invocations: Annotated[list[Invocation], Field(min_length=1)]

    @model_validator(mode="after")
    def validate_identities(self) -> Self:
        if len({item.request_id for item in self.invocations}) != len(self.invocations):
            raise ValueError("execution repeats a native request id")
        return self


class ExecutionEvidence(EvidenceModel, Generic[Values]):
    schema_version: SchemaVersion
    source: ExecutionSource
    values: Values


class EmbeddingValues(EvidenceModel):
    queries: dict[Text, list[float]]
    documents: dict[Text, list[float]]


class TranscriptionValues(EvidenceModel):
    transcripts: dict[Text, str]


class AlignedUnit(EvidenceModel):
    id: Text
    start_ms: Nonnegative | None
    end_ms: Nonnegative | None

    @model_validator(mode="after")
    def validate_boundaries(self) -> Self:
        if (self.start_ms is None) != (self.end_ms is None):
            raise ValueError("alignment must provide both boundaries or neither")
        if self.start_ms is not None and self.end_ms is not None and self.end_ms < self.start_ms:
            raise ValueError("alignment finishes before it starts")
        return self


class AlignmentValues(EvidenceModel):
    samples: dict[Text, list[AlignedUnit]]


class SynthesisArtifact(EvidenceModel):
    sample_id: Text
    audio: HashedFile


class SynthesisValues(EvidenceModel):
    artifacts: list[SynthesisArtifact]
