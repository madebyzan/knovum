from __future__ import annotations

from collections import defaultdict
from collections.abc import Callable, Iterable
from pathlib import Path
from typing import Any

from native_acceptance.common import (
    EvidenceError,
    percentile,
    require_keys,
    require_list,
    require_number,
    require_object,
    require_string,
    require_text,
    require_timestamp,
    resolve_hashed_file,
)
from native_acceptance.metrics import (
    character_tokens,
    cosine,
    edit_distance,
    ndcg_at_10,
    ranked_document_ids,
    recall_at_10,
    word_tokens,
)
from native_acceptance.schema import TTS_CRITERIA, load_native_output, validate_corpus, validate_provenance

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
            lambda section: _alignment(corpus["alignment"], section, results_path.parent, provenance),
        ),
        "tts": _evaluate_or_unavailable(
            "tts", capabilities["tts"], lambda section: _tts(corpus["tts"], section, results_path.parent, provenance)
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
    reference = load_native_output(
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
    candidate = load_native_output(
        section["candidateOutput"],
        base,
        f"{location}.candidateOutput",
        "embedding.encode",
        "embedding",
        provenance,
        {"native_worker"},
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
    baseline = load_native_output(
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
    candidate = load_native_output(
        section["candidateOutput"],
        base,
        f"{location}.candidateOutput",
        "speech.transcribe",
        "asr",
        provenance,
        {"native_worker"},
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


def _alignment(corpus: object, section: dict[str, Any], base: Path, provenance: dict[str, Any]) -> dict[str, Any]:
    location = "results.capabilities.alignment"
    require_keys(section, location, {"status", "candidateOutput"})
    values = load_native_output(
        section["candidateOutput"],
        base,
        f"{location}.candidateOutput",
        "speech.align",
        "alignment",
        provenance,
        {"native_worker"},
    )
    require_keys(values, "alignment values", {"samples"})
    output_samples = require_object(values["samples"], "alignment.values.samples")
    corpus_section = require_object(corpus, "corpus.alignment")
    samples = {item["id"]: item for item in require_list(corpus_section["samples"], "corpus.alignment.samples")}
    if output_samples.keys() != samples.keys():
        raise EvidenceError("alignment output identities differ from the fixed corpus")
    by_language: dict[str, dict[str, Any]] = {}
    all_errors: list[float] = []
    aligned_total = 0
    unit_total = 0
    for language in ("zh-CN", "en-US"):
        errors: list[float] = []
        aligned = 0
        total = 0
        for sample_id, sample in samples.items():
            if sample["language"] != language:
                continue
            expected_units = {unit["id"]: unit for unit in sample["units"]}
            actual_units = _alignment_units(output_samples[sample_id], expected_units.keys(), sample_id)
            total += len(expected_units)
            for unit_id, expected in expected_units.items():
                actual = actual_units[unit_id]
                if actual is not None:
                    aligned += 1
                    errors.extend((abs(actual[0] - expected["startMs"]), abs(actual[1] - expected["endMs"])))
        by_language[language] = _alignment_metrics(aligned, total, errors)
        aligned_total += aligned
        unit_total += total
        all_errors.extend(errors)
    overall = _alignment_metrics(aligned_total, unit_total, all_errors)
    thresholds = require_object(corpus_section["thresholds"], "corpus.alignment.thresholds")
    measured = [overall, *by_language.values()]
    passed = all(_alignment_passes(item, thresholds) for item in measured)
    return {
        "status": "passed" if passed else "failed",
        "thresholds": thresholds,
        "overall": overall,
        "byLanguage": by_language,
    }


def _alignment_units(raw: object, expected_ids: Iterable[str], sample_id: str) -> dict[str, tuple[float, float] | None]:
    items = require_list(raw, f"alignment.values.samples.{sample_id}")
    units: dict[str, tuple[float, float] | None] = {}
    for index, raw_item in enumerate(items):
        location = f"alignment.values.samples.{sample_id}[{index}]"
        item = require_object(raw_item, location)
        require_keys(item, location, {"id", "startMs", "endMs"})
        identifier = require_string(item["id"], f"{location}.id")
        if identifier in units:
            raise EvidenceError(f"alignment output repeats unit {identifier} for {sample_id}")
        start, end = item["startMs"], item["endMs"]
        if (start is None) != (end is None):
            raise EvidenceError(f"{location} must provide both boundaries or neither")
        if start is None:
            units[identifier] = None
        else:
            start_value = require_number(start, f"{location}.startMs")
            end_value = require_number(end, f"{location}.endMs")
            if start_value < 0.0 or end_value < start_value:
                raise EvidenceError(f"{location} has invalid timing")
            units[identifier] = (start_value, end_value)
    if units.keys() != set(expected_ids):
        raise EvidenceError(f"alignment units for {sample_id} differ from the fixed annotation")
    return units


def _alignment_metrics(aligned: int, total: int, errors: list[float]) -> dict[str, float | None]:
    if total == 0:
        raise EvidenceError("alignment corpus must contain annotated units for each language")
    return {
        "coverage": aligned / total,
        "medianBoundaryErrorMs": percentile(errors, 0.5) if errors else None,
        "p95BoundaryErrorMs": percentile(errors, 0.95) if errors else None,
    }


def _alignment_passes(metrics: dict[str, Any], thresholds: dict[str, Any]) -> bool:
    median = metrics["medianBoundaryErrorMs"]
    p95 = metrics["p95BoundaryErrorMs"]
    return (
        median is not None
        and p95 is not None
        and metrics["coverage"] >= thresholds["minimumCoverage"]
        and median <= thresholds["maximumMedianBoundaryErrorMs"]
        and p95 <= thresholds["maximumP95BoundaryErrorMs"]
    )


def _tts(corpus: object, section: dict[str, Any], base: Path, provenance: dict[str, Any]) -> dict[str, Any]:
    location = "results.capabilities.tts"
    require_keys(section, location, {"status", "candidateOutput", "blinding", "reviews"})
    values = load_native_output(
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
