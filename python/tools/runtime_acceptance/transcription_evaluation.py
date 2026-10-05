from __future__ import annotations

import re
import unicodedata
from collections.abc import Iterable
from pathlib import Path
from typing import Any

from runtime_acceptance.evidence import (
    EvidenceError,
    require_keys,
    require_list,
    require_object,
    require_text,
)
from runtime_acceptance.provenance import load_execution_evidence


def evaluate_transcription(
    corpus: object, section: dict[str, Any], base: Path, provenance: dict[str, Any]
) -> dict[str, Any]:
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
