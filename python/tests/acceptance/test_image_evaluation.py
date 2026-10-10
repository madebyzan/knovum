"""Synthetic contract evidence; these fixtures do not qualify a compiled image model."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import numpy as np
import pytest

from acceptance.evidence import EvidenceError, load_evidence
from acceptance.fixtures import execution_fixture, provenance_fixture, write_json
from acceptance.image_corpus import IMAGE_COVERAGE, ImageCapture, ImageCorpus
from acceptance.image_evaluation import evaluate_images


def image_files(directory: Path, capability: str = "repair") -> tuple[Path, Path]:
    from acceptance.evidence import sha256_file

    rgb = np.arange(48, dtype=np.uint8).reshape(1, 3, 4, 4)
    mask = np.zeros((1, 1, 4, 4), dtype=np.uint8)
    mask[:, :, 1:3, 1:3] = 255
    outputs: dict[str, Any] = {
        "repair": rgb.copy(),
        "upscale": np.repeat(np.repeat(rgb, 4, axis=2), 4, axis=3),
        "selection": mask,
        "depth": np.linspace(0, 1, 16, dtype=np.float32).reshape(1, 1, 4, 4),
    }
    output = outputs[capability]
    tensors = {}
    for name, pixels in (("rgb", rgb), ("mask", mask), ("output", output)):
        path = directory / f"{name}.npy"
        np.save(path, pixels)
        tensors[name] = {"path": path.name, "sha256": sha256_file(path)}
    corpus = {
        "schemaVersion": 1,
        "corpusId": "synthetic-unit-image",
        "corpusVersion": "1",
        "samples": [
            {
                "id": "sample",
                "original": tensors["rgb"],
                "renderedRgb": tensors["rgb"],
                "selectionMask": tensors["mask"],
                "license": "synthetic unit fixture",
                "sourceUrl": "https://example.invalid/synthetic-unit-fixture",
                "coverage": sorted(IMAGE_COVERAGE),
                "width": 4,
                "height": 4,
            }
        ],
        "profiles": [
            {
                "id": "model",
                "capability": capability,
                "model": {"specId": "image-model", "identityDigest": "a" * 64},
                "config": {"id": "image-profile", "parameters": {"threads": 4}},
                "sampleIds": ["sample"],
                "limits": {
                    "maximumLoadSeconds": 10,
                    "maximumWarmSeconds": 10,
                    "maximumPeakRssBytes": 1024,
                    "maximumCancelSeconds": 5,
                },
            }
        ],
    }
    reference = write_json(directory, "corpus.json", corpus)
    capture = {
        "schemaVersion": 1,
        "runId": "unit",
        "capturedAt": "2026-10-10T12:00:00Z",
        "appCommit": "a" * 40,
        "corpusSha256": reference.sha256,
        "workerDigest": "a" * 64,
        "runtimeDigest": "a" * 64,
        "target": "macos-x64",
        "device": provenance_fixture().device.model_dump(mode="json", by_alias=True),
        "models": [
            {
                "profileId": "model",
                "source": execution_fixture(f"image.{capability}", "image", {})["source"],
                "loadSeconds": 1,
                "peakRssBytes": 512,
                "artifacts": [
                    {
                        "sampleId": "sample",
                        "output": tensors["output"],
                        "warmSeconds": 1,
                        "cancelSeconds": 1,
                        "qualityPassed": True,
                        "reviewer": "synthetic",
                        "reviewNotes": "synthetic unit evidence",
                    }
                ],
            }
        ],
    }
    write_json(directory, "capture.json", capture)
    return directory / "corpus.json", directory / "capture.json"


def evaluate_paths(corpus: Path, capture: Path) -> dict[str, Any]:
    return evaluate_images(load_evidence(corpus, ImageCorpus), load_evidence(capture, ImageCapture), corpus, capture)


@pytest.mark.parametrize("capability", ["upscale", "repair", "selection", "depth"])
def test_image_shapes_and_frozen_evidence_chain(capability: str, tmp_path: Path) -> None:
    assert evaluate_paths(*image_files(tmp_path, capability))["status"] == "passed"


@pytest.mark.parametrize("mutation", ["corpus", "runtime", "worker", "model", "sample", "config"])
def test_image_capture_rejects_identity_changes(mutation: str, tmp_path: Path) -> None:
    corpus, capture = image_files(tmp_path)
    payload = json.loads(capture.read_bytes())
    if mutation == "corpus":
        payload["corpusSha256"] = "b" * 64
    elif mutation in {"runtime", "worker"}:
        payload["models"][0]["source"][f"{mutation}Digest"] = "b" * 64
    elif mutation == "sample":
        payload["models"][0]["artifacts"][0]["sampleId"] = "unknown"
    else:
        payload["models"][0]["source"]["modelSpecId" if mutation == "model" else "configId"] = "changed"
    capture.write_text(json.dumps(payload))
    with pytest.raises(EvidenceError):
        evaluate_paths(corpus, capture)


@pytest.mark.parametrize("mutation", ["missing-model", "source-adapter"])
def test_image_source_adapters_and_missing_captures_cannot_qualify_release(mutation: str, tmp_path: Path) -> None:
    corpus, capture = image_files(tmp_path)
    payload = json.loads(capture.read_bytes())
    if mutation == "missing-model":
        payload["models"] = []
    else:
        payload["models"][0]["source"]["kind"] = "official_python_reference"
        payload["models"][0]["source"]["workerDigest"] = None
    capture.write_text(json.dumps(payload))
    assert evaluate_paths(corpus, capture)["status"] == "unavailable"


@pytest.mark.parametrize("mutation", ["rss", "latency", "cancel", "quality"])
def test_image_performance_and_quality_limits_are_binding(mutation: str, tmp_path: Path) -> None:
    corpus, capture = image_files(tmp_path)
    payload = json.loads(capture.read_bytes())
    model = payload["models"][0]
    if mutation == "rss":
        model["peakRssBytes"] = 2048
    else:
        field = {"latency": "warmSeconds", "cancel": "cancelSeconds", "quality": "qualityPassed"}[mutation]
        model["artifacts"][0][field] = False if mutation == "quality" else 11
    capture.write_text(json.dumps(payload))
    assert evaluate_paths(corpus, capture)["status"] == "failed"


@pytest.mark.parametrize("capability", ["repair", "selection", "depth"])
def test_image_output_semantics_are_verified(capability: str, tmp_path: Path) -> None:
    from acceptance.evidence import sha256_file

    corpus, capture = image_files(tmp_path, capability)
    output = tmp_path / "output.npy"
    tensor = np.load(output, allow_pickle=False)
    tensor[0, 0, 0, 0] = 100 if capability != "depth" else float("nan")
    np.save(output, tensor)
    payload = json.loads(capture.read_bytes())
    payload["models"][0]["artifacts"][0]["output"]["sha256"] = sha256_file(output)
    capture.write_text(json.dumps(payload))
    with pytest.raises(EvidenceError):
        evaluate_paths(corpus, capture)


def test_image_profile_cannot_skip_difficult_categories(tmp_path: Path) -> None:
    corpus, _ = image_files(tmp_path)
    payload = json.loads(corpus.read_bytes())
    payload["samples"][0]["coverage"].remove("hair")
    corpus.write_text(json.dumps(payload))
    with pytest.raises(EvidenceError, match="image categor"):
        load_evidence(corpus, ImageCorpus)


def test_image_suite_runs_through_existing_report_command(tmp_path: Path) -> None:
    import subprocess
    import sys

    corpus, capture = image_files(tmp_path)
    output = tmp_path / "report.json"
    execution = subprocess.run(
        [
            sys.executable,
            "-I",
            "-B",
            str(Path(__file__).with_name("report.py")),
            "--suite",
            "images",
            "--corpus",
            str(corpus),
            "--capture",
            str(capture),
            "--output",
            str(output),
        ],
        check=False,
        capture_output=True,
        text=True,
        timeout=20,
    )
    assert execution.returncode == 0, execution.stderr
    assert json.loads(output.read_bytes())["suite"] == "images"
