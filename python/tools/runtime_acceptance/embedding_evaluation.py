from __future__ import annotations

import math
from collections.abc import Iterable
from pathlib import Path
from typing import Any

from runtime_acceptance.evidence import (
    EvidenceError,
    require_keys,
    require_list,
    require_number,
    require_object,
    require_string,
)
from runtime_acceptance.provenance import load_execution_evidence


def evaluate_embedding(
    corpus: object, section: dict[str, Any], base: Path, provenance: dict[str, Any]
) -> dict[str, Any]:
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


def cosine(left: list[float], right: list[float]) -> float:
    if not left or len(left) != len(right):
        raise EvidenceError("paired embedding vectors must have the same non-zero dimension")
    numerator = sum(a * b for a, b in zip(left, right, strict=True))
    left_norm = math.sqrt(sum(value * value for value in left))
    right_norm = math.sqrt(sum(value * value for value in right))
    if left_norm == 0.0 or right_norm == 0.0:
        raise EvidenceError("embedding vectors must have non-zero norm")
    result = numerator / (left_norm * right_norm)
    if not math.isfinite(result):
        raise EvidenceError("embedding cosine must be finite")
    return max(-1.0, min(1.0, result))


def ranked_document_ids(query: list[float], documents: dict[str, list[float]]) -> list[str]:
    return [
        document_id
        for document_id, _score in sorted(
            ((document_id, cosine(query, vector)) for document_id, vector in documents.items()),
            key=lambda item: (-item[1], item[0]),
        )
    ]


def recall_at_10(ranked: list[str], relevant: set[str]) -> float:
    denominator = len(relevant)
    if denominator == 0:
        raise EvidenceError("each retrieval query must have at least one relevant document")
    return len(relevant.intersection(ranked[:10])) / denominator


def ndcg_at_10(ranked: list[str], relevant: set[str]) -> float:
    actual = sum(1.0 / math.log2(index + 2) for index, item in enumerate(ranked[:10]) if item in relevant)
    ideal = sum(1.0 / math.log2(index + 2) for index in range(min(10, len(relevant))))
    return actual / ideal
