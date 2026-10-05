"""Synthetic unit evidence only; these bytes cannot qualify a native release."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from acceptance.capture_manifest import Provenance
from acceptance.contracts import HashedFile, VoiceScores
from acceptance.corpus import ASR_COVERAGE_TAGS


def write_json(directory: Path, name: str, values: dict[str, Any]) -> HashedFile:
    content = json.dumps(values).encode()
    (directory / name).write_bytes(content)
    return HashedFile(path=name, sha256=hashlib.sha256(content).hexdigest())


def provenance_fixture(corpus_digest: str = "a" * 64) -> Provenance:
    return Provenance.model_validate_json(
        json.dumps(
            {
                "appVersion": "unit-fixture",
                "appCommit": "a" * 40,
                "runtimeVersion": "unit-fixture",
                "runtimeDigest": "a" * 64,
                "target": "macos-arm64",
                "corpusSha256": corpus_digest,
                "models": {
                    name: {"specId": f"{name}-model", "identityDigest": "a" * 64}
                    for name in ("embedding", "asr", "alignment", "tts")
                },
                "configs": {
                    name: {"id": f"{name}-profile", "parameters": {"modelFamily": "whisper", "modelSize": "small"}}
                    for name in ("embedding", "asr", "alignment", "tts")
                },
                "device": {
                    "manufacturer": "fixture",
                    "model": "fixture",
                    "cpu": "fixture",
                    "physicalCores": 1,
                    "memoryBytes": 1024,
                    "os": "fixture",
                    "osVersion": "fixture",
                    "powerMode": "fixture",
                    "accelerator": "cpu",
                    "driver": None,
                },
            }
        )
    )


def execution_fixture(
    method: str, capability: str, values: dict[str, Any], kind: str = "bundled_worker"
) -> dict[str, Any]:
    return {
        "schemaVersion": 1,
        "source": {
            "kind": kind,
            "method": method,
            "runtimeDigest": "a" * 64,
            "modelSpecId": f"{capability}-model",
            "configId": f"{capability}-profile",
            "invocations": [
                {
                    "requestId": "unit-fixture",
                    "attemptEpoch": 1,
                    "startedAt": "2026-10-03T12:00:00Z",
                    "finishedAt": "2026-10-03T12:01:00Z",
                }
            ],
        },
        "values": values,
    }


def acceptance_files(directory: Path) -> tuple[Path, Path]:
    audio = directory / "fixture.wav"
    audio.write_bytes(b"unit fixture, not native qualification")
    reference = HashedFile(path=audio.name, sha256=hashlib.sha256(audio.read_bytes()).hexdigest()).model_dump()
    scores = dict.fromkeys((field.alias for field in VoiceScores.model_fields.values()), 4.0)
    speech_samples = [
        {
            "id": language,
            "language": language,
            "audio": reference,
            "durationMs": 600_000,
            "sampleRateHz": rate,
            "coverageTags": sorted(ASR_COVERAGE_TAGS),
            "referenceText": text,
        }
        for language, rate, text in (("zh-CN", 16000, "你好世界"), ("en-US", 48000, "It's a test"))
    ]
    speech_samples.append({**speech_samples[1], "id": "silence", "referenceText": ""})
    corpus: dict[str, Any] = {
        "schemaVersion": 1,
        "corpusId": "unit-fixture",
        "corpusVersion": "1",
        "description": "Synthetic unit evidence, not native qualification",
        "embedding": {
            "referenceProfile": {
                "modelSpecId": "embedding-model",
                "configId": "embedding-profile",
                "device": "cpu",
                "precision": "float32",
            },
            "documents": [
                {"id": kind, "kind": kind, "text": kind} for kind in ("document", "note", "code", "transcript")
            ],
            "queries": [{"id": "query", "text": "document", "relevantDocumentIds": ["document"]}],
        },
        "asr": {
            "baselineProfile": {
                "modelSpecId": "asr-model",
                "configId": "asr-profile",
                "modelFamily": "whisper",
                "modelSize": "small",
            },
            "samples": speech_samples,
        },
        "alignment": {
            "thresholds": {"minimumCoverage": 1, "maximumMedianBoundaryErrorMs": 20, "maximumP95BoundaryErrorMs": 50},
            "samples": [
                {
                    "id": item["id"],
                    "language": item["language"],
                    "audio": reference,
                    "durationMs": 1000,
                    "sampleRateHz": item["sampleRateHz"],
                    "units": [{"id": "unit", "text": "text", "startMs": 0, "endMs": 100}],
                }
                for item in speech_samples[:2]
            ],
        },
        "tts": {
            "thresholds": scores,
            "samples": [
                {"id": f"tts-{index}", "language": "zh-CN" if index % 2 else "en-US", "text": "unit fixture"}
                for index in range(30)
            ],
        },
    }
    corpus_reference = write_json(directory, "corpus.json", corpus)
    vectors = {
        "queries": {"query": [1.0, 0.0]},
        "documents": {
            "document": [1.0, 0.0],
            "note": [0.0, 1.0],
            "code": [-1.0, 0.0],
            "transcript": [0.0, -1.0],
        },
    }
    outputs = {
        "embedding-reference": execution_fixture("embedding.encode", "embedding", vectors, "official_python_reference"),
        "embedding-candidate": execution_fixture("embedding.encode", "embedding", vectors),
        "asr-baseline": execution_fixture(
            "speech.transcribe",
            "asr",
            {"transcripts": {item["id"]: item["referenceText"] for item in speech_samples}},
            "product_baseline",
        ),
        "asr-candidate": execution_fixture(
            "speech.transcribe", "asr", {"transcripts": {item["id"]: item["referenceText"] for item in speech_samples}}
        ),
        "alignment-candidate": execution_fixture(
            "speech.align",
            "alignment",
            {"samples": {item["id"]: [{"id": "unit", "startMs": 0, "endMs": 100}] for item in speech_samples[:2]}},
        ),
        "tts-candidate": execution_fixture(
            "speech.synthesize",
            "tts",
            {"artifacts": [{"sampleId": item["id"], "audio": reference} for item in corpus["tts"]["samples"]]},
        ),
    }
    references = {name: write_json(directory, f"{name}.json", values).model_dump() for name, values in outputs.items()}
    capture = {
        "schemaVersion": 1,
        "runId": "unit-fixture",
        "capturedAt": "2026-10-03T13:00:00Z",
        "provenance": provenance_fixture(corpus_reference.sha256).model_dump(mode="json", by_alias=True),
        "capabilities": {
            "embedding": {
                "status": "completed",
                "comparisonMode": "cpu_float32",
                "referenceOutput": references["embedding-reference"],
                "candidateOutput": references["embedding-candidate"],
            },
            "asr": {
                "status": "completed",
                "baselineOutput": references["asr-baseline"],
                "candidateOutput": references["asr-candidate"],
            },
            "alignment": {"status": "completed", "candidateOutput": references["alignment-candidate"]},
            "tts": {
                "status": "completed",
                "candidateOutput": references["tts-candidate"],
                "blinding": {"assignment": reference, "revealedAfterReviewsAt": "2026-10-03T13:00:00Z"},
                "reviews": [
                    {
                        "sampleId": item["id"],
                        "raterId": f"rater-{rater}",
                        "scores": scores,
                        "notes": None,
                        "reviewedAt": "2026-10-03T12:00:00Z",
                    }
                    for item in corpus["tts"]["samples"]
                    for rater in range(3)
                ],
            },
        },
    }
    write_json(directory, "capture.json", capture)
    return directory / "corpus.json", directory / "capture.json"
