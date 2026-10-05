from __future__ import annotations

from collections import defaultdict
from pathlib import Path
from typing import Any

from runtime_acceptance.common import (
    EvidenceError,
    require_keys,
    require_list,
    require_number,
    require_object,
    require_string,
    require_timestamp,
    resolve_hashed_file,
)
from runtime_acceptance.execution_evidence import load_execution_evidence
from runtime_acceptance.schema import TTS_CRITERIA


def evaluate_tts(corpus: object, section: dict[str, Any], base: Path, provenance: dict[str, Any]) -> dict[str, Any]:
    location = "results.capabilities.tts"
    require_keys(section, location, {"status", "candidateOutput", "blinding", "reviews"})
    values = load_execution_evidence(
        section["candidateOutput"],
        base,
        f"{location}.candidateOutput",
        "speech.synthesize",
        "tts",
        provenance,
        {"native_worker"},
    )
    require_keys(values, "TTS values", {"artifacts"})
    samples = {
        item["id"]: item for item in require_list(require_object(corpus, "corpus.tts")["samples"], "corpus.tts.samples")
    }
    artifacts = require_list(values["artifacts"], "TTS values.artifacts")
    artifact_ids: set[str] = set()
    for index, raw in enumerate(artifacts):
        item = require_object(raw, f"TTS values.artifacts[{index}]")
        require_keys(item, f"TTS values.artifacts[{index}]", {"sampleId", "audio"})
        sample_id = require_string(item["sampleId"], f"TTS values.artifacts[{index}].sampleId")
        resolve_hashed_file(item["audio"], base, f"TTS values.artifacts[{index}].audio")
        if sample_id in artifact_ids:
            raise EvidenceError(f"duplicate TTS artifact for {sample_id}")
        artifact_ids.add(sample_id)
    if artifact_ids != samples.keys():
        raise EvidenceError("TTS artifacts differ from the fixed corpus")
    blinding = require_object(section["blinding"], f"{location}.blinding")
    require_keys(blinding, f"{location}.blinding", {"assignment", "revealedAfterReviewsAt"})
    resolve_hashed_file(blinding["assignment"], base, f"{location}.blinding.assignment")
    revealed_at = require_timestamp(blinding["revealedAfterReviewsAt"], f"{location}.blinding.revealedAfterReviewsAt")
    reviews = require_list(section["reviews"], f"{location}.reviews")
    scores: dict[str, list[float]] = defaultdict(list)
    raters_by_sample: dict[str, set[str]] = defaultdict(set)
    for index, raw in enumerate(reviews):
        review_location = f"{location}.reviews[{index}]"
        review = require_object(raw, review_location)
        require_keys(review, review_location, {"sampleId", "raterId", "scores", "notes", "reviewedAt"})
        if require_timestamp(review["reviewedAt"], f"{review_location}.reviewedAt") >= revealed_at:
            raise EvidenceError(f"{review_location} was recorded after the blind assignment was revealed")
        sample_id = require_string(review["sampleId"], f"{review_location}.sampleId")
        if sample_id not in samples:
            raise EvidenceError(f"{review_location} refers to an unknown sample")
        rater_id = require_string(review["raterId"], f"{review_location}.raterId")
        if rater_id in raters_by_sample[sample_id]:
            raise EvidenceError(f"rater {rater_id} reviewed TTS sample {sample_id} more than once")
        raters_by_sample[sample_id].add(rater_id)
        if review["notes"] is not None:
            require_string(review["notes"], f"{review_location}.notes")
        review_scores = require_object(review["scores"], f"{review_location}.scores")
        require_keys(review_scores, f"{review_location}.scores", TTS_CRITERIA)
        for criterion in TTS_CRITERIA:
            score = require_number(review_scores[criterion], f"{review_location}.scores.{criterion}")
            if not 1.0 <= score <= 5.0:
                raise EvidenceError(f"{review_location}.scores.{criterion} must be between 1 and 5")
            scores[criterion].append(score)
    if any(len(raters_by_sample[sample_id]) < 3 for sample_id in samples):
        raise EvidenceError("every fixed TTS sample requires at least three independent blind reviews")
    means = {criterion: sum(values) / len(values) for criterion, values in scores.items()}
    thresholds = require_object(require_object(corpus, "corpus.tts")["thresholds"], "corpus.tts.thresholds")
    passed = all(means[criterion] >= thresholds[criterion] for criterion in TTS_CRITERIA)
    return {
        "status": "passed" if passed else "failed",
        "sampleCount": len(samples),
        "raterCount": len(set().union(*raters_by_sample.values())),
        "minimumRatersPerSample": min(map(len, raters_by_sample.values())),
        "meanScores": means,
        "thresholds": thresholds,
        "blinding": blinding,
    }
