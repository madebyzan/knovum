from __future__ import annotations

import json
from pathlib import Path

import pytest
from jiwer import process_characters, process_words

from acceptance.alignment_evaluation import alignment_metrics
from acceptance.capture_manifest import Unavailable
from acceptance.contracts import HashedFile
from acceptance.embedding_evaluation import cosine, ndcg_at_10, ranked_document_ids, recall_at_10
from acceptance.evaluator import evaluate_or_unavailable
from acceptance.evidence import EvidenceError, EvidenceUnavailable, resolve_hashed_file
from acceptance.execution import ExecutionEvidence, TranscriptionValues
from acceptance.fixtures import execution_fixture, provenance_fixture, write_json
from acceptance.provenance import load_execution_evidence
from acceptance.transcription_evaluation import character_tokens, word_tokens


def test_recall_uses_all_relevant_documents_as_denominator() -> None:
    assert recall_at_10([str(index) for index in range(10)], {str(index) for index in range(20)}) == 0.5
    assert ndcg_at_10([str(index) for index in range(10)], {str(index) for index in range(20)}) == 1.0


def test_cosine_detects_angle_changes_and_uses_stable_tie_order() -> None:
    assert cosine([1.0, 0.0], [1.0, 0.0]) == 1.0
    assert cosine([1.0, 0.0], [0.0, 1.0]) == 0.0
    assert ranked_document_ids([1.0, 0.0], {"z": [1.0, 0.0], "a": [1.0, 0.0]}) == ["a", "z"]
    for left, right in (([0.0], [1.0]), ([1.0], [1.0, 1.0]), ([float("nan")], [1.0])):
        with pytest.raises(EvidenceError):
            cosine(left, right)


def test_normalization_and_jiwer_count_substitution_deletion_and_silence_insertions() -> None:
    chars = process_characters("".join(character_tokens("你好，世界！")), "".join(character_tokens("你好世间")))
    assert chars.substitutions == 1
    words = process_words(" ".join(word_tokens("It's a test.")), " ".join(word_tokens("it's test again")))
    assert words.deletions + words.insertions + words.substitutions == 2
    assert process_words("", " ".join(word_tokens("hallucinated on silence"))).insertions == 3
    assert character_tokens("Ａ，Ｂ！") == ["a", "b"]


def test_alignment_percentiles_preserve_linear_interpolation_and_no_alignment_state() -> None:
    assert alignment_metrics(2, 2, [0.0, 10.0, 20.0, 30.0]) == pytest.approx(
        {
            "coverage": 1.0,
            "medianBoundaryErrorMs": 15.0,
            "p95BoundaryErrorMs": 28.5,
        }
    )
    assert alignment_metrics(0, 2, []) == {
        "coverage": 0.0,
        "medianBoundaryErrorMs": None,
        "p95BoundaryErrorMs": None,
    }


def test_hash_bound_evidence_is_rejected_after_change(tmp_path: Path) -> None:
    reference = write_json(tmp_path, "output.json", {"unit": "fixture"})
    assert resolve_hashed_file(reference, tmp_path, "output") == tmp_path / "output.json"
    (tmp_path / reference.path).write_text("changed")
    with pytest.raises(EvidenceError, match="hash mismatch"):
        resolve_hashed_file(reference, tmp_path, "output")


def test_missing_and_escaping_evidence_cannot_pass(tmp_path: Path) -> None:
    with pytest.raises(EvidenceUnavailable):
        resolve_hashed_file(HashedFile(path="absent.wav", sha256="a" * 64), tmp_path, "audio")
    with pytest.raises(EvidenceError, match="below"):
        resolve_hashed_file(HashedFile(path="../escaped.wav", sha256="a" * 64), tmp_path, "audio")
    actual = tmp_path / "actual"
    actual.write_bytes(b"evidence")
    (tmp_path / "alias").symlink_to(actual)
    with pytest.raises(EvidenceError, match="symbolic link"):
        resolve_hashed_file(HashedFile(path="alias", sha256="a" * 64), tmp_path, "audio")


def test_unavailable_capability_is_reported_without_calculating_a_pass() -> None:
    assert evaluate_or_unavailable(
        Unavailable(status="unavailable", reason="native output absent"),
        lambda _: pytest.fail("missing output must not be scored"),
    ) == {"status": "unavailable", "reason": "native output absent"}


@pytest.mark.parametrize(
    ("change", "message"),
    [
        ({}, None),
        ({"method": "embedding.encode"}, "required native execution path"),
        ({"workerDigest": "b" * 64}, "Worker digest differs"),
        ({"runtimeDigest": "b" * 64}, "runtime digest differs"),
        ({"modelSpecId": "different-model"}, "frozen candidate model"),
        ({"configId": "different-config"}, "frozen candidate config"),
        ({"invocations": []}, "invocations"),
    ],
)
def test_execution_provenance_rejects_outputs_from_other_runs(
    tmp_path: Path, change: dict[str, object], message: str | None
) -> None:
    values = execution_fixture("speech.transcribe", "asr", {"transcripts": {"sample": "hello"}})
    values["source"].update(change)
    reference = write_json(tmp_path, "asr.json", values)

    def load() -> TranscriptionValues:
        return load_execution_evidence(
            reference,
            tmp_path,
            "asr",
            "speech.transcribe",
            "asr",
            provenance_fixture(),
            "bundled_worker",
            ExecutionEvidence[TranscriptionValues],
        )

    if message is None:
        assert load().transcripts == {"sample": "hello"}
    else:
        with pytest.raises(EvidenceError, match=message):
            load()


@pytest.mark.parametrize(
    "change",
    [
        {"attemptEpoch": True},
        {"attemptEpoch": -1},
        {"attemptEpoch": 1.0},
        {"startedAt": "2026-10-03T12:00:00"},
        {"finishedAt": "2026-10-03T11:00:00Z"},
    ],
)
def test_invocation_contract_rejects_coercion_and_invalid_times(change: dict[str, object]) -> None:
    from pydantic import ValidationError

    values = execution_fixture("speech.transcribe", "asr", {"transcripts": {"sample": "hello"}})
    values["source"]["invocations"][0].update(change)
    with pytest.raises(ValidationError):
        ExecutionEvidence[TranscriptionValues].model_validate_json(json.dumps(values))


@pytest.mark.parametrize("number", [float("nan"), float("inf"), float("-inf")])
def test_configuration_rejects_non_json_numbers_inside_nested_parameters(number: float) -> None:
    from pydantic import ValidationError

    from acceptance.capture_manifest import ExecutionConfiguration

    with pytest.raises(ValidationError):
        ExecutionConfiguration.model_validate_json(json.dumps({"id": "profile", "parameters": {"nested": [number]}}))
