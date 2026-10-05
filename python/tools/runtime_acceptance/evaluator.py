from __future__ import annotations

from collections.abc import Callable, Iterable
from pathlib import Path
from typing import Any

from runtime_acceptance.alignment_evaluation import evaluate_alignment
from runtime_acceptance.evidence import (
    EvidenceError,
    require_keys,
    require_list,
    require_number,
    require_object,
    require_string,
    require_text,
)
from runtime_acceptance.execution_evidence import load_execution_evidence
from runtime_acceptance.metrics import (
    character_tokens,
    cosine,
    edit_distance,
    ndcg_at_10,
    ranked_document_ids,
    recall_at_10,
    word_tokens,
)
from runtime_acceptance.provenance import validate_provenance
from runtime_acceptance.schema import validate_corpus
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
            lambda section: _embedding(corpus["embedding"], section, results_path.parent, provenance),
        ),
        "asr": _evaluate_or_unavailable(
            "asr", capabilities["asr"], lambda section: _asr(corpus["asr"], section, results_path.parent, provenance)
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


def _embedding(corpus: object, section: dict[str, Any], base: Path, provenance: dict[str, Any]) -> dict[str, Any]:
    location = "results.capabilities.embedding"
    require_keys(section, location, {"status", "comparisonMode", "referenceOutput", "candidateOutput"})
    mode = require_string(section["comparisonMode"], f"{location}.comparisonMode")
    if mode not in {"cpu_float32", "release_profile"}:
        raise EvidenceError(f"{location}.comparisonMode must be cpu_float32 or release_profile")
    reference_profile = require_object(
        require_object(corpus, "corpus.embedding")["referenceProfile"], "corpus.embedding.referenceProfile"
    )
    reference = load_execution_evidence(
        section["referenceOutput"],
        base,
        f"{location}.referenceOutput",
        "embedding.encode",
        "embedding",
        provenance,
        {"official_python_reference"},
        reference_profile["modelSpecId"],
        reference_profile["configId"],
    )
    candidate = load_execution_evidence(
        section["candidateOutput"],
        base,
        f"{location}.candidateOutput",
        "embedding.encode",
        "embedding",
        provenance,
        {"bundled_worker"},
    )
    corpus_section = require_object(corpus, "corpus.embedding")
    query_relevance = {
        item["id"]: set(item["relevantDocumentIds"])
        for item in require_list(corpus_section["queries"], "corpus.embedding.queries")
    }
    document_ids = {item["id"] for item in require_list(corpus_section["documents"], "corpus.embedding.documents")}
    reference_queries, reference_documents = _embedding_vectors(
        reference, query_relevance.keys(), document_ids, "reference"
    )
    candidate_queries, candidate_documents = _embedding_vectors(
        candidate, query_relevance.keys(), document_ids, "candidate"
    )
    paired = [cosine(reference_queries[item], candidate_queries[item]) for item in sorted(query_relevance)] + [
        cosine(reference_documents[item], candidate_documents[item]) for item in sorted(document_ids)
    ]
    reference_recall, reference_ndcg = _retrieval(reference_queries, reference_documents, query_relevance)
    candidate_recall, candidate_ndcg = _retrieval(candidate_queries, candidate_documents, query_relevance)
    recall_drop = reference_recall - candidate_recall
    ndcg_drop = reference_ndcg - candidate_ndcg
    cosine_passed = mode != "cpu_float32" or min(paired) >= 0.9999
    passed = cosine_passed and recall_drop <= 0.01 and ndcg_drop <= 0.01
    return {
        "status": "passed" if passed else "failed",
        "comparisonMode": mode,
        "minimumPairedCosine": min(paired),
        "cosineThreshold": 0.9999 if mode == "cpu_float32" else None,
        "referenceRecallAt10": reference_recall,
        "candidateRecallAt10": candidate_recall,
        "recallAt10Drop": recall_drop,
        "referenceNdcgAt10": reference_ndcg,
        "candidateNdcgAt10": candidate_ndcg,
        "ndcgAt10Drop": ndcg_drop,
        "maximumRetrievalDrop": 0.01,
    }


def _embedding_vectors(
    values: dict[str, Any], query_ids: Iterable[str], document_ids: set[str], label: str
) -> tuple[dict[str, list[float]], dict[str, list[float]]]:
    require_keys(values, f"{label} embedding values", {"queries", "documents"})
    queries = _vectors(values["queries"], f"{label}.queries")
    documents = _vectors(values["documents"], f"{label}.documents")
    if queries.keys() != set(query_ids) or documents.keys() != document_ids:
        raise EvidenceError(f"{label} embedding output identities differ from the fixed corpus")
    dimensions = {len(vector) for vector in [*queries.values(), *documents.values()]}
    if len(dimensions) != 1 or next(iter(dimensions)) == 0:
        raise EvidenceError(f"{label} embedding output has inconsistent dimensions")
    return queries, documents


def _vectors(raw: object, location: str) -> dict[str, list[float]]:
    source = require_object(raw, location)
    vectors: dict[str, list[float]] = {}
    for identifier, raw_vector in source.items():
        vector = [
            require_number(value, f"{location}.{identifier}")
            for value in require_list(raw_vector, f"{location}.{identifier}")
        ]
        vectors[identifier] = vector
    return vectors


def _retrieval(
    queries: dict[str, list[float]], documents: dict[str, list[float]], relevance: dict[str, set[str]]
) -> tuple[float, float]:
    recalls: list[float] = []
    ndcgs: list[float] = []
    for query_id, relevant in relevance.items():
        ranked = ranked_document_ids(queries[query_id], documents)
        recalls.append(recall_at_10(ranked, relevant))
        ndcgs.append(ndcg_at_10(ranked, relevant))
    return sum(recalls) / len(recalls), sum(ndcgs) / len(ndcgs)


def _asr(corpus: object, section: dict[str, Any], base: Path, provenance: dict[str, Any]) -> dict[str, Any]:
    location = "results.capabilities.asr"
    require_keys(section, location, {"status", "baselineOutput", "candidateOutput"})
    baseline_profile = require_object(
        require_object(corpus, "corpus.asr")["baselineProfile"], "corpus.asr.baselineProfile"
    )
    parameters = provenance["configs"]["asr"]["parameters"]
    if any(parameters.get(key) != baseline_profile[key] for key in ("modelFamily", "modelSize")):
        raise EvidenceError("ASR comparison requires the preregistered model family and size")
    baseline = load_execution_evidence(
        section["baselineOutput"],
        base,
        f"{location}.baselineOutput",
        "speech.transcribe",
        "asr",
        provenance,
        {"product_baseline"},
        baseline_profile["modelSpecId"],
        baseline_profile["configId"],
    )
    candidate = load_execution_evidence(
        section["candidateOutput"],
        base,
        f"{location}.candidateOutput",
        "speech.transcribe",
        "asr",
        provenance,
        {"bundled_worker"},
    )
    samples = require_list(require_object(corpus, "corpus.asr")["samples"], "corpus.asr.samples")
    references = {item["id"]: item for item in samples}
    baseline_texts = _transcripts(baseline, references.keys(), "baseline")
    candidate_texts = _transcripts(candidate, references.keys(), "candidate")
    metrics: dict[str, Any] = {}
    passed = True
    for language, tokenizer, metric_name in (("zh-CN", character_tokens, "cer"), ("en-US", word_tokens, "wer")):
        reference_count = 0
        baseline_errors = 0
        candidate_errors = 0
        for sample_id, sample in references.items():
            if sample["language"] == language:
                reference_units = tokenizer(sample["referenceText"])
                baseline_units = tokenizer(baseline_texts[sample_id])
                candidate_units = tokenizer(candidate_texts[sample_id])
                reference_count += len(reference_units)
                baseline_errors += edit_distance(reference_units, baseline_units)
                candidate_errors += edit_distance(reference_units, candidate_units)
        if not reference_count:
            raise EvidenceError(f"ASR corpus has no scored units for {language}")
        baseline_rate = baseline_errors / reference_count
        candidate_rate = candidate_errors / reference_count
        regression = candidate_rate - baseline_rate
        metrics[metric_name] = {
            "baseline": baseline_rate,
            "candidate": candidate_rate,
            "regression": regression,
            "maximumRegression": 0.01,
        }
        passed = passed and regression <= 0.01
    return {"status": "passed" if passed else "failed", **metrics}


def _transcripts(values: dict[str, Any], expected_ids: Iterable[str], label: str) -> dict[str, str]:
    require_keys(values, f"{label} ASR values", {"transcripts"})
    transcripts = require_object(values["transcripts"], f"{label}.transcripts")
    if transcripts.keys() != set(expected_ids):
        raise EvidenceError(f"{label} ASR output identities differ from the fixed corpus")
    return {
        identifier: require_text(text, f"{label}.transcripts.{identifier}") for identifier, text in transcripts.items()
    }
