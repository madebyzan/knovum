from __future__ import annotations

import re
import unicodedata
from pathlib import Path
from typing import Any

from jiwer import process_characters, process_words

from acceptance.capture_manifest import Provenance, TranscriptionEvidence
from acceptance.corpus import TranscriptionCorpus
from acceptance.evidence import EvidenceError
from acceptance.execution import ExecutionEvidence, TranscriptionValues
from acceptance.provenance import load_execution_evidence


def evaluate_transcription(
    corpus: TranscriptionCorpus, section: TranscriptionEvidence, base: Path, provenance: Provenance
) -> dict[str, Any]:
    profile = corpus.baseline_profile
    parameters = provenance.configs.asr.parameters
    if parameters.get("modelFamily") != profile.model_family or parameters.get("modelSize") != profile.model_size:
        raise EvidenceError("ASR comparison requires the preregistered model family and size")
    baseline = load_execution_evidence(
        section.baseline_output,
        base,
        "asr.baselineOutput",
        "speech.transcribe",
        "asr",
        provenance,
        "product_baseline",
        ExecutionEvidence[TranscriptionValues],
        profile.model_spec_id,
        profile.config_id,
    )
    candidate = load_execution_evidence(
        section.candidate_output,
        base,
        "asr.candidateOutput",
        "speech.transcribe",
        "asr",
        provenance,
        "bundled_worker",
        ExecutionEvidence[TranscriptionValues],
    )
    sample_ids = {item.id for item in corpus.samples}
    if baseline.transcripts.keys() != sample_ids or candidate.transcripts.keys() != sample_ids:
        raise EvidenceError("ASR output identities differ from the fixed corpus")
    metrics: dict[str, Any] = {}
    for language, tokenizer, separator, processor, name in (
        ("zh-CN", character_tokens, "", process_characters, "cer"),
        ("en-US", word_tokens, " ", process_words, "wer"),
    ):
        samples = [item for item in corpus.samples if item.language == language]
        references = [separator.join(tokenizer(item.reference_text)) for item in samples]
        baseline_texts = [separator.join(tokenizer(baseline.transcripts[item.id])) for item in samples]
        candidate_texts = [separator.join(tokenizer(candidate.transcripts[item.id])) for item in samples]
        baseline_counts = processor(references, baseline_texts)
        candidate_counts = processor(references, candidate_texts)
        reference_count = baseline_counts.hits + baseline_counts.substitutions + baseline_counts.deletions
        if not reference_count:
            raise EvidenceError(f"ASR corpus has no scored units for {language}")
        baseline_rate = (
            baseline_counts.substitutions + baseline_counts.deletions + baseline_counts.insertions
        ) / reference_count
        candidate_rate = (
            candidate_counts.substitutions + candidate_counts.deletions + candidate_counts.insertions
        ) / reference_count
        metrics[name] = {
            "baseline": baseline_rate,
            "candidate": candidate_rate,
            "regression": candidate_rate - baseline_rate,
            "maximumRegression": 0.01,
        }
    passed = all(item["regression"] <= 0.01 for item in metrics.values())
    return {"status": "passed" if passed else "failed", **metrics}


def character_tokens(text: str) -> list[str]:
    normalized = unicodedata.normalize("NFKC", text).casefold()
    return [character for character in normalized if character.isalnum()]


def word_tokens(text: str) -> list[str]:
    normalized = unicodedata.normalize("NFKC", text).casefold()
    return re.findall(r"[^\W_]+(?:'[^\W_]+)?", normalized, flags=re.UNICODE)
