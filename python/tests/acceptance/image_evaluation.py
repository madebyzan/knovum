"""Evaluate frozen image tensors and measured worker evidence without claiming unmeasured quality."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from acceptance.contracts import HashedFile
from acceptance.evidence import EvidenceError, resolve_hashed_file, sha256_file
from acceptance.image_corpus import ImageCapture, ImageCorpus, ImageProfile, ImageSample

MAX_TENSOR_BYTES = 256 * 1024 * 1024


def _tensor(reference: HashedFile, base: Path, location: str) -> NDArray[Any]:
    path = resolve_hashed_file(reference, base, location)
    if path.stat().st_size > MAX_TENSOR_BYTES:
        raise EvidenceError(f"{location} exceeds the tensor byte budget")
    try:
        tensor = np.load(path, allow_pickle=False, mmap_mode="r")
    except (ValueError, OSError) as error:
        raise EvidenceError(f"{location} is not a numeric tensor: {error}") from error
    if not isinstance(tensor, np.ndarray) or not np.isfinite(tensor).all():
        raise EvidenceError(f"{location} must contain finite tensor values")
    return tensor


def _output(
    profile: ImageProfile, sample: ImageSample, source: NDArray[Any], mask: NDArray[Any], output: NDArray[Any]
) -> None:
    height, width = sample.height, sample.width
    expected = {
        "upscale": ((1, 3, height * 4, width * 4), np.dtype("uint8")),
        "repair": ((1, 3, height, width), np.dtype("uint8")),
        "selection": ((1, 1, height, width), np.dtype("uint8")),
        "depth": ((1, 1, height, width), np.dtype("float32")),
    }[profile.capability]
    if output.shape != expected[0] or output.dtype != expected[1]:
        raise EvidenceError(f"{profile.id}/{sample.id} output shape or dtype differs from its contract")
    if profile.capability == "repair":
        protected = np.repeat(mask == 0, 3, axis=1)
        if not np.array_equal(output[protected], source[protected]):
            raise EvidenceError("image repair changed unselected source pixels")
    if profile.capability == "selection" and not np.isin(output, [0, 255]).all():
        raise EvidenceError("image selection must be a binary mask")
    if profile.capability == "depth" and (output.min() == output.max() or output.min() < 0 or output.max() > 1):
        raise EvidenceError("relative depth must be a nonconstant normalized field")


def evaluate_images(
    corpus: ImageCorpus, capture: ImageCapture, corpus_path: Path, capture_path: Path
) -> dict[str, Any]:
    if capture.corpus_sha256 != sha256_file(corpus_path):
        raise EvidenceError("image capture does not match the supplied corpus bytes")
    sources = {}
    for sample in corpus.samples:
        resolve_hashed_file(sample.original, corpus_path.parent, f"{sample.id}.original")
        rgb = _tensor(sample.rendered_rgb, corpus_path.parent, f"{sample.id}.renderedRgb")
        mask = _tensor(sample.selection_mask, corpus_path.parent, f"{sample.id}.selectionMask")
        if rgb.shape != (1, 3, sample.height, sample.width) or rgb.dtype != np.uint8:
            raise EvidenceError("image corpus RGB must match its declared dimensions")
        if mask.shape != (1, 1, sample.height, sample.width) or mask.dtype != np.uint8:
            raise EvidenceError("image corpus mask must match its rendered RGB")
        if not np.isin(mask, [0, 255]).all() or not np.any(mask == 255):
            raise EvidenceError("image corpus requires a nonempty binary selected region")
        sources[sample.id] = (sample, rgb, mask)
    profiles = {profile.id: profile for profile in corpus.profiles}
    model_ids = [model.profile_id for model in capture.models]
    if len(set(model_ids)) != len(model_ids) or not set(model_ids).issubset(profiles):
        raise EvidenceError("image capture repeats or introduces an unregistered model profile")
    evaluations: dict[str, Any] = {}
    for model in capture.models:
        profile = profiles[model.profile_id]
        identity = model.source
        if (
            identity.runtime_digest != capture.runtime_digest
            or identity.model_spec_id != profile.model.spec_id
            or identity.config_id != profile.config.id
            or identity.method != f"image.{profile.capability}"
        ):
            raise EvidenceError("image execution differs from its frozen profile")
        if identity.kind == "bundled_worker" and identity.worker_digest != capture.worker_digest:
            raise EvidenceError("image execution Worker digest differs from the capture")
        sample_ids = [artifact.sample_id for artifact in model.artifacts]
        if len(set(sample_ids)) != len(sample_ids) or set(sample_ids) != set(profile.sample_ids):
            raise EvidenceError("image execution must cover its complete registered corpus")
        limits = profile.limits
        failures = []
        if model.load_seconds > limits.maximum_load_seconds:
            failures.append("cold load exceeds its profile")
        if model.peak_rss_bytes > limits.maximum_peak_rss_bytes:
            failures.append("peak RSS exceeds its profile")
        for artifact in model.artifacts:
            sample, rgb, mask = sources[artifact.sample_id]
            output = _tensor(artifact.output, capture_path.parent, f"{profile.id}/{sample.id}.output")
            _output(profile, sample, rgb, mask, output)
            if artifact.warm_seconds > limits.maximum_warm_seconds:
                failures.append(f"{sample.id}: warm inference exceeds its profile")
            if artifact.cancel_seconds > limits.maximum_cancel_seconds:
                failures.append(f"{sample.id}: in-flight cancellation exceeds its profile")
            if not artifact.quality_passed:
                failures.append(f"{sample.id}: visual quality review failed")
        status = "failed" if failures else "passed" if identity.kind == "bundled_worker" else "unavailable"
        evaluations[profile.id] = {
            "status": status,
            "failures": failures,
            "executionKind": identity.kind,
            "loadSeconds": model.load_seconds,
            "peakRssBytes": model.peak_rss_bytes,
            "artifacts": [artifact.model_dump(mode="json", by_alias=True) for artifact in model.artifacts],
        }
    for profile_id in profiles.keys() - evaluations.keys():
        evaluations[profile_id] = {"status": "unavailable", "reason": "no compiled model capture"}
    statuses = {result["status"] for result in evaluations.values()}
    return {
        "schemaVersion": 1,
        "status": "failed" if "failed" in statuses else "unavailable" if "unavailable" in statuses else "passed",
        "suite": "images",
        "runId": capture.run_id,
        "target": capture.target,
        "corpusSha256": capture.corpus_sha256,
        "runtimeDigest": capture.runtime_digest,
        "workerDigest": capture.worker_digest,
        "device": capture.device.model_dump(mode="json", by_alias=True),
        "models": evaluations,
    }
