from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any

import pytest

from acceptance.capture_manifest import CaptureManifest
from acceptance.corpus import Corpus
from acceptance.evaluator import evaluate
from acceptance.evidence import EvidenceError, load_evidence
from acceptance.fixtures import acceptance_files, write_json


def evaluate_files(corpus: Path, capture: Path) -> dict[str, Any]:
    return evaluate(load_evidence(corpus, Corpus), load_evidence(capture, CaptureManifest), corpus, capture)


def change_output(capture: Path, capability: str, values: dict[str, Any]) -> None:
    manifest = json.loads(capture.read_text())
    output = manifest["capabilities"][capability]["candidateOutput"]["path"]
    payload = json.loads((capture.parent / output).read_text())
    payload["values"] = values
    manifest["capabilities"][capability]["candidateOutput"] = write_json(capture.parent, output, payload).model_dump()
    capture.write_text(json.dumps(manifest))


def run_report(directory: Path) -> tuple[int, dict[str, Any]]:
    report = directory / "report.json"
    execution = subprocess.run(
        [
            sys.executable,
            "-I",
            "-B",
            "-X",
            "utf8",
            str(Path(__file__).with_name("report.py")),
            "--corpus",
            str(directory / "corpus.json"),
            "--capture",
            str(directory / "capture.json"),
            "--output",
            str(report),
        ],
        capture_output=True,
        text=True,
        timeout=20,
    )
    assert report.is_file(), execution.stderr
    return execution.returncode, json.loads(report.read_text())


def test_complete_synthetic_evidence_chain_passes_and_preserves_all_metrics(tmp_path: Path) -> None:
    corpus, capture = acceptance_files(tmp_path)
    report = evaluate_files(corpus, capture)
    assert report["status"] == "passed"
    assert report["capabilities"]["embedding"]["minimumPairedCosine"] == 1
    assert report["capabilities"]["embedding"]["candidateRecallAt10"] == 1
    assert report["capabilities"]["asr"]["cer"]["candidate"] == 0
    assert report["capabilities"]["asr"]["wer"]["candidate"] == 0
    assert report["capabilities"]["alignment"]["overall"]["coverage"] == 1
    assert report["capabilities"]["tts"]["minimumRatersPerSample"] == 3
    assert report["provenance"]["runtimeDigest"] == "a" * 64
    code, persisted = run_report(tmp_path)
    assert code == 0
    assert persisted == report


def test_silence_hallucinations_are_micro_aggregated_into_english_wer(tmp_path: Path) -> None:
    corpus, capture = acceptance_files(tmp_path)
    change_output(
        capture,
        "asr",
        {"transcripts": {"zh-CN": "你好世界", "en-US": "It's a test", "silence": "hallucinated on silence"}},
    )
    metrics = evaluate_files(corpus, capture)["capabilities"]["asr"]
    assert metrics["cer"]["candidate"] == 0
    assert metrics["wer"]["candidate"] == 1  # Three insertions / three reference words, not a per-sample mean.
    assert metrics["status"] == "failed"
    assert run_report(tmp_path)[0] == 1


def test_alignment_language_threshold_cannot_be_hidden_by_overall_average(tmp_path: Path) -> None:
    corpus, capture = acceptance_files(tmp_path)
    change_output(
        capture,
        "alignment",
        {
            "samples": {
                "zh-CN": [{"id": "unit", "startMs": 30, "endMs": 130}],
                "en-US": [{"id": "unit", "startMs": 0, "endMs": 100}],
            }
        },
    )
    report = evaluate_files(corpus, capture)["capabilities"]["alignment"]
    assert report["overall"]["medianBoundaryErrorMs"] == 15
    assert report["byLanguage"]["zh-CN"]["medianBoundaryErrorMs"] == 30
    assert report["status"] == "failed"


@pytest.mark.parametrize(
    ("change", "message"),
    [
        ("duplicate-rater", "more than once"),
        ("revealed", "revealed"),
        ("missing-review", "three independent"),
        ("unknown-sample", "unknown sample"),
    ],
)
def test_tts_requires_three_distinct_reviews_before_reveal(tmp_path: Path, change: str, message: str) -> None:
    corpus, capture = acceptance_files(tmp_path)
    payload = json.loads(capture.read_text())
    reviews = payload["capabilities"]["tts"]["reviews"]
    if change == "duplicate-rater":
        reviews[0]["raterId"] = reviews[1]["raterId"]
    elif change == "revealed":
        reviews[0]["reviewedAt"] = "2026-10-03T13:00:00Z"
    elif change == "unknown-sample":
        reviews[0]["sampleId"] = "unknown"
    else:
        reviews.pop()
    capture.write_text(json.dumps(payload))
    with pytest.raises(EvidenceError, match=message):
        evaluate_files(corpus, capture)


def test_unavailable_cannot_hide_failed_scores(tmp_path: Path) -> None:
    corpus, capture = acceptance_files(tmp_path)
    manifest = json.loads(capture.read_text())
    manifest["capabilities"]["tts"] = {"status": "unavailable", "reason": "native audio absent"}
    capture.write_text(json.dumps(manifest))
    assert evaluate_files(corpus, capture)["status"] == "unavailable"
    assert run_report(tmp_path)[0] == 2
    change_output(capture, "asr", {"transcripts": {"zh-CN": "错", "en-US": "wrong", "silence": ""}})
    assert evaluate_files(corpus, capture)["status"] == "failed"
    assert run_report(tmp_path)[0] == 1


def test_isolated_report_distinguishes_missing_and_invalid_evidence(tmp_path: Path) -> None:
    code, report = run_report(tmp_path)
    assert code == 2 and report["status"] == "unavailable"
    acceptance_files(tmp_path)
    (tmp_path / "capture.json").write_text("{broken")
    code, report = run_report(tmp_path)
    assert code == 1 and report["status"] == "failed"


@pytest.mark.parametrize(
    ("path", "value"),
    [
        (("schemaVersion",), True),
        (("extra",), "unknown"),
        (("embedding", "referenceProfile", "device"), "gpu"),
        (("embedding", "queries", 0, "relevantDocumentIds"), ["unknown"]),
        (("asr", "samples", 0, "durationMs"), "600000"),
        (("asr", "samples", 0, "sampleRateHz"), True),
        (("alignment", "samples", 0, "units", 0, "endMs"), -1),
        (("tts", "thresholds", "pronunciation"), 6),
        (("tts", "samples", 0, "language"), "de-DE"),
    ],
)
def test_corpus_contract_rejects_unknown_fields_coercion_and_invalid_annotations(
    tmp_path: Path,
    path: tuple[str | int, ...],
    value: object,
) -> None:
    corpus, _ = acceptance_files(tmp_path)
    payload = json.loads(corpus.read_text())
    cursor = payload
    for part in path[:-1]:
        cursor = cursor[part]
    cursor[path[-1]] = value
    corpus.write_text(json.dumps(payload))
    with pytest.raises(EvidenceError):
        load_evidence(corpus, Corpus)


@pytest.mark.parametrize("change", ["tags", "rates", "duration", "ids", "tts-count", "document-kinds"])
def test_corpus_enforces_preregistered_coverage(tmp_path: Path, change: str) -> None:
    corpus, _ = acceptance_files(tmp_path)
    payload = json.loads(corpus.read_text())
    if change in {"tags", "rates", "duration"}:
        for sample in payload["asr"]["samples"]:
            if change == "tags":
                sample["coverageTags"] = []
            elif change == "rates":
                sample["sampleRateHz"] = 16000
            else:
                sample["durationMs"] = 1000
    elif change == "ids":
        payload["asr"]["samples"][1]["id"] = payload["asr"]["samples"][0]["id"]
    elif change == "tts-count":
        payload["tts"]["samples"].pop()
    else:
        payload["embedding"]["documents"].pop()
    corpus.write_text(json.dumps(payload))
    with pytest.raises(EvidenceError):
        load_evidence(corpus, Corpus)


def test_candidate_vectors_require_finite_numbers_and_complete_identities(tmp_path: Path) -> None:
    corpus, capture = acceptance_files(tmp_path)
    change_output(capture, "embedding", {"queries": {"query": [float("nan"), 0]}, "documents": {}})
    with pytest.raises(EvidenceError, match="finite"):
        evaluate_files(corpus, capture)
    change_output(capture, "embedding", {"queries": {"other-query": [1, 0]}, "documents": {}})
    with pytest.raises(EvidenceError, match="identities"):
        evaluate_files(corpus, capture)


def test_changed_corpus_bytes_invalidate_candidate_provenance(tmp_path: Path) -> None:
    corpus, capture = acceptance_files(tmp_path)
    corpus.write_text(corpus.read_text() + "\n")
    with pytest.raises(EvidenceError, match="corpus bytes"):
        evaluate_files(corpus, capture)
