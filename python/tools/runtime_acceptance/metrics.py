from __future__ import annotations

import math
import re
import unicodedata

from runtime_acceptance.common import EvidenceError


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


def character_tokens(text: str) -> list[str]:
    normalized = unicodedata.normalize("NFKC", text).casefold()
    return [character for character in normalized if character.isalnum()]


def word_tokens(text: str) -> list[str]:
    normalized = unicodedata.normalize("NFKC", text).casefold()
    return re.findall(r"[^\W_]+(?:'[^\W_]+)?", normalized, flags=re.UNICODE)


def edit_distance(reference: list[str], hypothesis: list[str]) -> int:
    previous = list(range(len(hypothesis) + 1))
    for reference_index, reference_item in enumerate(reference, start=1):
        current = [reference_index]
        for hypothesis_index, hypothesis_item in enumerate(hypothesis, start=1):
            current.append(
                min(
                    current[-1] + 1,
                    previous[hypothesis_index] + 1,
                    previous[hypothesis_index - 1] + (reference_item != hypothesis_item),
                )
            )
        previous = current
    return previous[-1]
