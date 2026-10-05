from __future__ import annotations

from collections import defaultdict
from pathlib import Path
from typing import Any

import numpy as np

from acceptance.corpus import SynthesisCorpus
from acceptance.evidence import EvidenceError, resolve_hashed_file
from acceptance.execution import ExecutionEvidence, SynthesisValues
from acceptance.provenance import load_execution_evidence
from acceptance.results import Provenance, SynthesisResult


def evaluate_tts(
    corpus: SynthesisCorpus, section: SynthesisResult, base: Path, provenance: Provenance
) -> dict[str, Any]:
    values = load_execution_evidence(
        section.candidate_output,
        base,
        "tts.candidateOutput",
        "speech.synthesize",
        "tts",
        provenance,
        "bundled_worker",
        ExecutionEvidence[SynthesisValues],
    )
    sample_ids = {item.id for item in corpus.samples}
    artifact_ids: set[str] = set()
    for artifact in values.artifacts:
        resolve_hashed_file(artifact.audio, base, f"tts.artifacts.{artifact.sample_id}.audio")
        if artifact.sample_id in artifact_ids:
            raise EvidenceError(f"duplicate TTS artifact for {artifact.sample_id}")
        artifact_ids.add(artifact.sample_id)
    if artifact_ids != sample_ids:
        raise EvidenceError("TTS artifacts differ from the fixed corpus")
    resolve_hashed_file(section.blinding.assignment, base, "tts.blinding.assignment")
    scores: dict[str, list[float]] = defaultdict(list)
    raters_by_sample: dict[str, set[str]] = defaultdict(set)
    for review in section.reviews:
        if review.reviewed_at >= section.blinding.revealed_after_reviews_at:
            raise EvidenceError("TTS review was recorded after the blind assignment was revealed")
        if review.sample_id not in sample_ids:
            raise EvidenceError("TTS review refers to an unknown sample")
        if review.rater_id in raters_by_sample[review.sample_id]:
            raise EvidenceError(f"rater {review.rater_id} reviewed TTS sample {review.sample_id} more than once")
        raters_by_sample[review.sample_id].add(review.rater_id)
        for criterion, score in review.scores.model_dump(by_alias=True).items():
            scores[criterion].append(score)
    if any(len(raters_by_sample[sample_id]) < 3 for sample_id in sample_ids):
        raise EvidenceError("every fixed TTS sample requires at least three independent blind reviews")
    means = {criterion: float(np.mean(values)) for criterion, values in scores.items()}
    thresholds = corpus.thresholds.model_dump(by_alias=True)
    passed = all(means[criterion] >= thresholds[criterion] for criterion in thresholds)
    return {
        "status": "passed" if passed else "failed",
        "sampleCount": len(sample_ids),
        "raterCount": len(set().union(*raters_by_sample.values())),
        "minimumRatersPerSample": min(map(len, raters_by_sample.values())),
        "meanScores": means,
        "thresholds": thresholds,
        "blinding": section.blinding.model_dump(mode="json", by_alias=True),
    }
