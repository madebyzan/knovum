"""Native inference capture manifest and capability evidence."""

from __future__ import annotations

import json
from typing import Annotated, Generic, Literal, TypeVar

from pydantic import AwareDatetime, Field, JsonValue, field_validator

from acceptance.contracts import Capability, EvidenceModel, HashedFile, SchemaVersion, Sha256, Text, VoiceScores

Record = TypeVar("Record")


class CapabilityRecords(EvidenceModel, Generic[Record]):
    embedding: Record
    asr: Record
    alignment: Record
    tts: Record

    def select(self, capability: Capability) -> Record:
        return {"embedding": self.embedding, "asr": self.asr, "alignment": self.alignment, "tts": self.tts}[capability]


class ModelIdentity(EvidenceModel):
    spec_id: Text
    identity_digest: Sha256


class ExecutionConfiguration(EvidenceModel):
    id: Text
    parameters: dict[str, JsonValue]

    @field_validator("parameters")
    @classmethod
    def validate_json_numbers(cls, parameters: dict[str, JsonValue]) -> dict[str, JsonValue]:
        # JsonValue's recursive validator does not inherit allow_inf_nan from its parent model.
        json.dumps(parameters, allow_nan=False)
        return parameters


class Device(EvidenceModel):
    manufacturer: Text
    model: Text
    cpu: Text
    physical_cores: Annotated[int, Field(gt=0)]
    memory_bytes: Annotated[int, Field(gt=0)]
    os: Text
    os_version: Text
    power_mode: Text
    accelerator: Text
    driver: Text | None


class Provenance(EvidenceModel):
    app_version: Text
    app_commit: Annotated[str, Field(pattern=r"^[0-9a-f]{40}$")]
    runtime_version: Text
    runtime_digest: Sha256
    target: Literal["macos-arm64", "windows-x64"]
    models: CapabilityRecords[ModelIdentity]
    configs: CapabilityRecords[ExecutionConfiguration]
    corpus_sha256: Sha256
    device: Device


class Unavailable(EvidenceModel):
    status: Literal["unavailable"]
    reason: Text


class EmbeddingEvidence(EvidenceModel):
    status: Literal["completed"]
    comparison_mode: Literal["cpu_float32", "release_profile"]
    reference_output: HashedFile
    candidate_output: HashedFile


class TranscriptionEvidence(EvidenceModel):
    status: Literal["completed"]
    baseline_output: HashedFile
    candidate_output: HashedFile


class AlignmentEvidence(EvidenceModel):
    status: Literal["completed"]
    candidate_output: HashedFile


class Blinding(EvidenceModel):
    assignment: HashedFile
    revealed_after_reviews_at: AwareDatetime


class VoiceReview(EvidenceModel):
    sample_id: Text
    rater_id: Text
    scores: VoiceScores
    notes: Text | None
    reviewed_at: AwareDatetime


class SynthesisEvidence(EvidenceModel):
    status: Literal["completed"]
    candidate_output: HashedFile
    blinding: Blinding
    reviews: list[VoiceReview]


class CapabilityEvidence(EvidenceModel):
    embedding: Annotated[EmbeddingEvidence | Unavailable, Field(discriminator="status")]
    asr: Annotated[TranscriptionEvidence | Unavailable, Field(discriminator="status")]
    alignment: Annotated[AlignmentEvidence | Unavailable, Field(discriminator="status")]
    tts: Annotated[SynthesisEvidence | Unavailable, Field(discriminator="status")]


class CaptureManifest(EvidenceModel):
    schema_version: SchemaVersion
    run_id: Text
    captured_at: AwareDatetime
    provenance: Provenance
    capabilities: CapabilityEvidence
