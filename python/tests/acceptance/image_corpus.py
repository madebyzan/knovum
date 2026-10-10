"""Preregistered image inputs, model identities and target-specific qualification evidence."""

from __future__ import annotations

from typing import Annotated, Literal, Self

from pydantic import AwareDatetime, Field, model_validator

from acceptance.capture_manifest import Device, ExecutionConfiguration, ModelIdentity
from acceptance.contracts import EvidenceModel, HashedFile, Nonnegative, Positive, SchemaVersion, Sha256, Text
from acceptance.execution import ExecutionSource

ImageCapability = Literal["upscale", "repair", "selection", "depth"]
ImageCoverage = Literal[
    "hair", "product_edges", "transparent", "text", "anime", "repeated_texture", "large_selection", "thin", "exif"
]
IMAGE_COVERAGE = {
    "hair",
    "product_edges",
    "transparent",
    "text",
    "anime",
    "repeated_texture",
    "large_selection",
    "thin",
    "exif",
}


class ImageSample(EvidenceModel):
    id: Text
    original: HashedFile
    rendered_rgb: HashedFile
    license: Text
    source_url: Annotated[str, Field(pattern=r"^https://")]
    coverage: Annotated[list[ImageCoverage], Field(min_length=1)]
    width: Annotated[int, Field(gt=0, le=10000)]
    height: Annotated[int, Field(gt=0, le=10000)]
    selection_mask: HashedFile


class ImageLimits(EvidenceModel):
    maximum_load_seconds: Positive
    maximum_warm_seconds: Positive
    maximum_peak_rss_bytes: Annotated[int, Field(gt=0)]
    maximum_cancel_seconds: Positive


class ImageProfile(EvidenceModel):
    id: Text
    capability: ImageCapability
    model: ModelIdentity
    config: ExecutionConfiguration
    sample_ids: Annotated[list[Text], Field(min_length=1)]
    limits: ImageLimits


class ImageCorpus(EvidenceModel):
    schema_version: SchemaVersion
    corpus_id: Text
    corpus_version: Text
    samples: Annotated[list[ImageSample], Field(min_length=1)]
    profiles: Annotated[list[ImageProfile], Field(min_length=1)]

    @model_validator(mode="after")
    def validate_coverage(self) -> Self:
        ids = {sample.id for sample in self.samples}
        if len(ids) != len(self.samples) or len({profile.id for profile in self.profiles}) != len(self.profiles):
            raise ValueError("image corpus identities must be unique")
        if {tag for sample in self.samples for tag in sample.coverage} != IMAGE_COVERAGE:
            raise ValueError("image corpus must cover every preregistered image category")
        for profile in self.profiles:
            if len(set(profile.sample_ids)) != len(profile.sample_ids) or not set(profile.sample_ids).issubset(ids):
                raise ValueError("image profile must identify unique corpus samples")
            tags = {tag for sample in self.samples if sample.id in profile.sample_ids for tag in sample.coverage}
            if tags != IMAGE_COVERAGE:
                raise ValueError("each image model must cover every image category")
        return self


class ImageArtifact(EvidenceModel):
    sample_id: Text
    output: HashedFile
    warm_seconds: Nonnegative
    cancel_seconds: Nonnegative
    quality_passed: bool
    reviewer: Text
    review_notes: Text


class ImageModelCapture(EvidenceModel):
    profile_id: Text
    source: ExecutionSource
    load_seconds: Nonnegative
    peak_rss_bytes: Annotated[int, Field(gt=0)]
    artifacts: Annotated[list[ImageArtifact], Field(min_length=1)]


class ImageCapture(EvidenceModel):
    schema_version: SchemaVersion
    run_id: Text
    captured_at: AwareDatetime
    app_commit: Annotated[str, Field(pattern=r"^[0-9a-f]{40}$")]
    corpus_sha256: Sha256
    worker_digest: Sha256
    runtime_digest: Sha256
    target: Literal["macos-arm64", "macos-x64", "windows-x64"]
    device: Device
    models: list[ImageModelCapture]
