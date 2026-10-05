from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

import pytest
from runtime_acceptance.embedding_evaluation import cosine, ndcg_at_10, ranked_document_ids, recall_at_10
from runtime_acceptance.evaluator import _evaluate_or_unavailable
from runtime_acceptance.evidence import EvidenceError, EvidenceUnavailable, resolve_hashed_file
from runtime_acceptance.transcription_evaluation import character_tokens, edit_distance, word_tokens


def test_recall_uses_all_relevant_documents_as_denominator() -> None:
    assert recall_at_10([str(index) for index in range(10)], {str(index) for index in range(20)}) == 0.5
    assert ndcg_at_10([str(index) for index in range(10)], {str(index) for index in range(20)}) == 1.0


def test_numeric_baseline_detects_angle_changes_and_uses_stable_tie_order() -> None:
    assert cosine([1.0, 0.0], [1.0, 0.0]) == 1.0
    assert cosine([1.0, 0.0], [0.0, 1.0]) == 0.0
    assert ranked_document_ids([1.0, 0.0], {"z": [1.0, 0.0], "a": [1.0, 0.0]}) == ["a", "z"]
    with pytest.raises(EvidenceError):
        cosine([0.0], [1.0])
    with pytest.raises(EvidenceError):
        cosine([1.0], [1.0, 1.0])


def test_cer_and_wer_count_insertions_deletions_and_substitutions() -> None:
    assert edit_distance(character_tokens("你好，世界！"), character_tokens("你好世间")) == 1
    assert edit_distance(word_tokens("It's a test."), word_tokens("it's test again")) == 2
    assert edit_distance([], word_tokens("hallucinated on silence")) == 3


def test_hash_bound_evidence_is_rejected_after_change(tmp_path: Path) -> None:
    path = tmp_path / "audio.wav"
    path.write_bytes(b"fixed evidence")
    reference = {"path": "audio.wav", "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    assert resolve_hashed_file(reference, tmp_path, "audio") == path
    path.write_bytes(b"changed")
    with pytest.raises(EvidenceError, match="hash mismatch"):
        resolve_hashed_file(reference, tmp_path, "audio")


def test_missing_and_escaping_evidence_cannot_pass(tmp_path: Path) -> None:
    with pytest.raises(EvidenceUnavailable):
        resolve_hashed_file({"path": "absent.wav", "sha256": "a" * 64}, tmp_path, "audio")
    with pytest.raises(EvidenceError, match="below"):
        resolve_hashed_file({"path": "../escaped.wav", "sha256": "a" * 64}, tmp_path, "audio")
    actual = tmp_path / "actual"
    actual.write_bytes(b"evidence")
    (tmp_path / "alias").symlink_to(actual)
    with pytest.raises(EvidenceError, match="symbolic link"):
        resolve_hashed_file({"path": "alias", "sha256": "a" * 64}, tmp_path, "audio")


def test_unavailable_capability_is_reported_without_calculating_a_pass() -> None:
    report = _evaluate_or_unavailable(
        "asr",
        {"status": "unavailable", "reason": "native output absent"},
        lambda _: pytest.fail("missing output must not be scored"),
    )
    assert report == {"status": "unavailable", "reason": "native output absent"}


def test_isolated_report_command_returns_unavailable_for_missing_native_inputs(tmp_path: Path) -> None:
    output = tmp_path / "report.json"
    result = subprocess.run(
        [
            sys.executable,
            "-I",
            "-B",
            "-X",
            "utf8",
            str(Path(__file__).resolve().parents[2] / "tools/acceptance_report.py"),
            "--corpus",
            str(tmp_path / "corpus.json"),
            "--results",
            str(tmp_path / "results.json"),
            "--output",
            str(output),
        ],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 2, result.stderr
    assert json.loads(output.read_text())["status"] == "unavailable"


def test_tts_requires_three_distinct_reviews_before_blind_reveal(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    from runtime_acceptance.schema import TTS_CRITERIA
    from runtime_acceptance.tts_evaluation import evaluate_tts

    artifact = tmp_path / "fixture.wav"
    artifact.write_bytes(b"test fixture, not native qualification")
    reference = {"path": artifact.name, "sha256": hashlib.sha256(artifact.read_bytes()).hexdigest()}
    samples = [{"id": f"sample-{index}"} for index in range(30)]
    monkeypatch.setattr(
        "runtime_acceptance.tts_evaluation.load_execution_evidence",
        lambda *args: {"artifacts": [{"sampleId": sample["id"], "audio": reference} for sample in samples]},
    )
    corpus = {"samples": samples, "thresholds": dict.fromkeys(TTS_CRITERIA, 3.5)}
    reviews = [
        {
            "sampleId": sample["id"],
            "raterId": f"rater-{rater}",
            "reviewedAt": "2026-10-03T12:00:00Z",
            "notes": None,
            "scores": dict.fromkeys(TTS_CRITERIA, 4),
        }
        for sample in samples
        for rater in range(3)
    ]
    section = {
        "status": "completed",
        "candidateOutput": {},
        "blinding": {"assignment": reference, "revealedAfterReviewsAt": "2026-10-03T13:00:00Z"},
        "reviews": reviews,
    }
    assert evaluate_tts(corpus, section, tmp_path, {})["status"] == "passed"
    reviews[0]["raterId"] = reviews[1]["raterId"]
    with pytest.raises(EvidenceError, match="more than once"):
        evaluate_tts(corpus, section, tmp_path, {})
    reviews[0]["raterId"] = "rater-0"
    reviews[0]["reviewedAt"] = "2026-10-03T14:00:00Z"
    with pytest.raises(EvidenceError, match="revealed"):
        evaluate_tts(corpus, section, tmp_path, {})
    reviews[0]["reviewedAt"] = "2026-10-03T12:00:00Z"
    reviews.pop()
    with pytest.raises(EvidenceError, match="three independent"):
        evaluate_tts(corpus, section, tmp_path, {})


@pytest.mark.parametrize(
    ("change", "message"),
    [
        ({}, None),
        ({"method": "embedding.encode"}, "required native execution path"),
        ({"runtimeDigest": "b" * 64}, "runtime digest differs"),
        ({"modelSpecId": "different-model"}, "frozen candidate model"),
        ({"configId": "different-config"}, "frozen candidate config"),
        ({"invocations": []}, "invocation identities"),
    ],
)
def test_execution_provenance_rejects_outputs_from_other_runs(
    tmp_path: Path, change: dict[str, object], message: str | None
) -> None:
    from runtime_acceptance.provenance import load_execution_evidence

    source = {
        "kind": "bundled_worker",
        "method": "speech.transcribe",
        "runtimeDigest": "a" * 64,
        "modelSpecId": "whisper-model",
        "configId": "asr-profile",
        "invocations": [
            {
                "requestId": "request",
                "attemptEpoch": 1,
                "startedAt": "2026-10-03T12:00:00Z",
                "finishedAt": "2026-10-03T12:01:00Z",
            }
        ],
        **change,
    }
    output = tmp_path / "asr.json"
    payload = json.dumps(
        {"schemaVersion": 1, "source": source, "values": {"transcripts": {"sample": "hello"}}}
    ).encode()
    output.write_bytes(payload)
    reference = {"path": output.name, "sha256": hashlib.sha256(payload).hexdigest()}
    provenance = {
        "runtimeDigest": "a" * 64,
        "models": {"asr": {"specId": "whisper-model"}},
        "configs": {"asr": {"id": "asr-profile"}},
    }
    if message is None:
        assert load_execution_evidence(
            reference, tmp_path, "asr", "speech.transcribe", "asr", provenance, {"bundled_worker"}
        ) == {"transcripts": {"sample": "hello"}}
    else:
        with pytest.raises(EvidenceError, match=message):
            load_execution_evidence(
                reference, tmp_path, "asr", "speech.transcribe", "asr", provenance, {"bundled_worker"}
            )
