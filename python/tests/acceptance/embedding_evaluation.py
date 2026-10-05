from __future__ import annotations

import math
from pathlib import Path
from typing import Any

import numpy as np

from acceptance.corpus import EmbeddingCorpus
from acceptance.evidence import EvidenceError
from acceptance.execution import EmbeddingValues, ExecutionEvidence
from acceptance.provenance import load_execution_evidence
from acceptance.results import EmbeddingResult, Provenance


def evaluate_embedding(
    corpus: EmbeddingCorpus, section: EmbeddingResult, base: Path, provenance: Provenance
) -> dict[str, Any]:
    profile = corpus.reference_profile
    reference = load_execution_evidence(
        section.reference_output,
        base,
        "embedding.referenceOutput",
        "embedding.encode",
        "embedding",
        provenance,
        "official_python_reference",
        ExecutionEvidence[EmbeddingValues],
        profile.model_spec_id,
        profile.config_id,
    )
    candidate = load_execution_evidence(
        section.candidate_output,
        base,
        "embedding.candidateOutput",
        "embedding.encode",
        "embedding",
        provenance,
        "bundled_worker",
        ExecutionEvidence[EmbeddingValues],
    )
    relevance = {item.id: set(item.relevant_document_ids) for item in corpus.queries}
    document_ids = {item.id for item in corpus.documents}
    for label, values in (("reference", reference), ("candidate", candidate)):
        if values.queries.keys() != relevance.keys() or values.documents.keys() != document_ids:
            raise EvidenceError(f"{label} embedding output identities differ from the fixed corpus")
        dimensions = {len(vector) for vector in [*values.queries.values(), *values.documents.values()]}
        if len(dimensions) != 1 or 0 in dimensions:
            raise EvidenceError(f"{label} embedding output has inconsistent dimensions")
    paired = [cosine(reference.queries[item], candidate.queries[item]) for item in sorted(relevance)] + [
        cosine(reference.documents[item], candidate.documents[item]) for item in sorted(document_ids)
    ]
    reference_recall, reference_ndcg = _retrieval(reference, relevance)
    candidate_recall, candidate_ndcg = _retrieval(candidate, relevance)
    recall_drop = reference_recall - candidate_recall
    ndcg_drop = reference_ndcg - candidate_ndcg
    mode = section.comparison_mode
    passed = (mode != "cpu_float32" or min(paired) >= 0.9999) and recall_drop <= 0.01 and ndcg_drop <= 0.01
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


def _retrieval(values: EmbeddingValues, relevance: dict[str, set[str]]) -> tuple[float, float]:
    recalls: list[float] = []
    ndcgs: list[float] = []
    for query_id, relevant in relevance.items():
        ranked = ranked_document_ids(values.queries[query_id], values.documents)
        recalls.append(recall_at_10(ranked, relevant))
        ndcgs.append(ndcg_at_10(ranked, relevant))
    return float(np.mean(recalls)), float(np.mean(ndcgs))


def cosine(left: list[float], right: list[float]) -> float:
    if not left or len(left) != len(right):
        raise EvidenceError("paired embedding vectors must have the same non-zero dimension")
    try:
        with np.errstate(over="raise", invalid="raise", divide="raise"):
            vectors = np.asarray([left, right], dtype=np.float64)
            norms = np.linalg.norm(vectors, axis=1)
            if not np.all(np.isfinite(vectors)) or not np.all(np.isfinite(norms)) or np.any(norms == 0):
                raise EvidenceError("embedding vectors must be finite and have non-zero norm")
            normalized = vectors / norms[:, None]
            return float(np.clip(np.dot(normalized[0], normalized[1]), -1.0, 1.0))
    except FloatingPointError as error:
        raise EvidenceError("embedding cosine must be finite") from error


def ranked_document_ids(query: list[float], documents: dict[str, list[float]]) -> list[str]:
    return [
        document_id
        for document_id, _score in sorted(
            ((document_id, cosine(query, vector)) for document_id, vector in documents.items()),
            key=lambda item: (-item[1], item[0]),
        )
    ]


def recall_at_10(ranked: list[str], relevant: set[str]) -> float:
    if not relevant:
        raise EvidenceError("each retrieval query must have at least one relevant document")
    return len(relevant.intersection(ranked[:10])) / len(relevant)


def ndcg_at_10(ranked: list[str], relevant: set[str]) -> float:
    if not relevant:
        raise EvidenceError("each retrieval query must have at least one relevant document")
    actual = sum(1.0 / math.log2(index + 2) for index, item in enumerate(ranked[:10]) if item in relevant)
    ideal = sum(1.0 / math.log2(index + 2) for index in range(min(10, len(relevant))))
    return actual / ideal
