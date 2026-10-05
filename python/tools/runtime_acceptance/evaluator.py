from __future__ import annotations

from collections.abc import Callable
from pathlib import Path
from typing import Any

from runtime_acceptance.alignment_evaluation import evaluate_alignment
from runtime_acceptance.embedding_evaluation import evaluate_embedding
from runtime_acceptance.evidence import (
    EvidenceError,
    require_keys,
    require_object,
    require_string,
)
from runtime_acceptance.provenance import validate_provenance
from runtime_acceptance.schema import validate_corpus
from runtime_acceptance.transcription_evaluation import evaluate_transcription
from runtime_acceptance.tts_evaluation import evaluate_tts

CAPABILITIES = {"embedding", "asr", "alignment", "tts"}


def evaluate(corpus: dict[str, Any], results: dict[str, Any], corpus_path: Path, results_path: Path) -> dict[str, Any]:
    validate_corpus(corpus, corpus_path)
    provenance = validate_provenance(results, corpus_path)
    capabilities = require_object(results["capabilities"], "results.capabilities")
    if capabilities.keys() != CAPABILITIES:
        raise EvidenceError("results.capabilities must explicitly cover embedding, asr, alignment, and tts")
    evaluations = {
        "embedding": _evaluate_or_unavailable(
            "embedding",
            capabilities["embedding"],
            lambda section: evaluate_embedding(corpus["embedding"], section, results_path.parent, provenance),
        ),
        "asr": _evaluate_or_unavailable(
            "asr",
            capabilities["asr"],
            lambda section: evaluate_transcription(corpus["asr"], section, results_path.parent, provenance),
        ),
        "alignment": _evaluate_or_unavailable(
            "alignment",
            capabilities["alignment"],
            lambda section: evaluate_alignment(corpus["alignment"], section, results_path.parent, provenance),
        ),
        "tts": _evaluate_or_unavailable(
            "tts",
            capabilities["tts"],
            lambda section: evaluate_tts(corpus["tts"], section, results_path.parent, provenance),
        ),
    }
    statuses = {item["status"] for item in evaluations.values()}
    overall = "failed" if "failed" in statuses else "unavailable" if "unavailable" in statuses else "passed"
    return {
        "schemaVersion": 1,
        "status": overall,
        "runId": results["runId"],
        "capturedAt": results["capturedAt"],
        "corpus": {"id": corpus["corpusId"], "version": corpus["corpusVersion"], "sha256": provenance["corpusSha256"]},
        "provenance": provenance,
        "capabilities": evaluations,
    }


def _evaluate_or_unavailable(
    name: str, raw: object, evaluator: Callable[[dict[str, Any]], dict[str, Any]]
) -> dict[str, Any]:
    section = require_object(raw, f"results.capabilities.{name}")
    status = section.get("status")
    if status == "unavailable":
        require_keys(section, f"results.capabilities.{name}", {"status", "reason"})
        return {
            "status": "unavailable",
            "reason": require_string(section["reason"], f"results.capabilities.{name}.reason"),
        }
    if status != "completed":
        raise EvidenceError(f"results.capabilities.{name}.status must be completed or unavailable")
    return evaluator(section)
