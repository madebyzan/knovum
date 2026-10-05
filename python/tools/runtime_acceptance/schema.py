from __future__ import annotations

import re
from pathlib import Path
from typing import Any

from runtime_acceptance.evidence import (
    EvidenceError,
    require_keys,
    require_list,
    require_number,
    require_object,
    require_string,
    require_text,
    resolve_hashed_file,
)

COMMIT_PATTERN = re.compile(r"[0-9a-f]{40}")
LANGUAGES = {"zh-CN", "en-US"}
TARGETS = {"macos-arm64", "windows-x64"}
TTS_CRITERIA = {
    "pronunciation",
    "naturalness",
    "voiceConsistency",
    "referenceSimilarity",
    "emotionControl",
}
ASR_COVERAGE_TAGS = {
    "mixed_language",
    "proper_noun",
    "numbers",
    "silence",
    "noise",
    "speaker_overlap",
    "long_recording",
}


def validate_corpus(corpus: dict[str, Any], path: Path) -> None:
    require_keys(
        corpus,
        "corpus",
        {"schemaVersion", "corpusId", "corpusVersion", "description", "embedding", "asr", "alignment", "tts"},
    )
    if corpus["schemaVersion"] != 1:
        raise EvidenceError("corpus.schemaVersion must be 1")
    require_string(corpus["corpusId"], "corpus.corpusId")
    require_string(corpus["corpusVersion"], "corpus.corpusVersion")
    require_string(corpus["description"], "corpus.description")
    _validate_embedding_corpus(require_object(corpus["embedding"], "corpus.embedding"))
    _validate_asr_corpus(require_object(corpus["asr"], "corpus.asr"), path.parent)
    _validate_alignment_corpus(require_object(corpus["alignment"], "corpus.alignment"), path.parent)
    _validate_tts_corpus(require_object(corpus["tts"], "corpus.tts"))


def _validate_embedding_corpus(section: dict[str, Any]) -> None:
    require_keys(section, "corpus.embedding", {"referenceProfile", "documents", "queries"})
    profile = require_object(section["referenceProfile"], "corpus.embedding.referenceProfile")
    require_keys(profile, "corpus.embedding.referenceProfile", {"modelSpecId", "configId", "device", "precision"})
    require_string(profile["modelSpecId"], "corpus.embedding.referenceProfile.modelSpecId")
    require_string(profile["configId"], "corpus.embedding.referenceProfile.configId")
    if profile["device"] != "cpu" or profile["precision"] != "float32":
        raise EvidenceError("embedding reference profile must be cpu/float32")
    documents = require_list(section["documents"], "corpus.embedding.documents")
    if not documents:
        raise EvidenceError("corpus.embedding.documents must not be empty")
    document_ids: set[str] = set()
    document_kinds: set[str] = set()
    allowed_kinds = {"document", "note", "code", "transcript"}
    for index, raw in enumerate(documents):
        location = f"corpus.embedding.documents[{index}]"
        document = require_object(raw, location)
        require_keys(document, location, {"id", "kind", "text"})
        document_id = require_string(document["id"], f"{location}.id")
        kind = require_string(document["kind"], f"{location}.kind")
        if document_id in document_ids or kind not in allowed_kinds:
            raise EvidenceError(f"{location} has a duplicate id or unsupported kind")
        document_ids.add(document_id)
        document_kinds.add(kind)
        require_string(document["text"], f"{location}.text")
    if document_kinds != allowed_kinds:
        raise EvidenceError("embedding corpus must cover document, note, code, and transcript retrieval")
    queries = require_list(section["queries"], "corpus.embedding.queries")
    if not queries:
        raise EvidenceError("corpus.embedding.queries must not be empty")
    query_ids: set[str] = set()
    for index, raw in enumerate(queries):
        location = f"corpus.embedding.queries[{index}]"
        query = require_object(raw, location)
        require_keys(query, location, {"id", "text", "relevantDocumentIds"})
        query_id = require_string(query["id"], f"{location}.id")
        if query_id in query_ids:
            raise EvidenceError(f"duplicate embedding query id: {query_id}")
        query_ids.add(query_id)
        require_string(query["text"], f"{location}.text")
        relevant = {
            require_string(value, f"{location}.relevantDocumentIds")
            for value in require_list(query["relevantDocumentIds"], f"{location}.relevantDocumentIds")
        }
        if not relevant or not relevant.issubset(document_ids):
            raise EvidenceError(f"{location} must identify existing relevant documents")


def _validate_asr_corpus(section: dict[str, Any], base: Path) -> None:
    require_keys(section, "corpus.asr", {"baselineProfile", "samples"})
    profile = require_object(section["baselineProfile"], "corpus.asr.baselineProfile")
    require_keys(profile, "corpus.asr.baselineProfile", {"modelSpecId", "configId", "modelFamily", "modelSize"})
    for key in profile:
        require_string(profile[key], f"corpus.asr.baselineProfile.{key}")
    samples = require_list(section["samples"], "corpus.asr.samples")
    _require_languages(samples, "corpus.asr.samples")
    seen: set[str] = set()
    coverage_tags: set[str] = set()
    sample_rates: set[int] = set()
    has_ten_minute_sample = False
    for index, raw in enumerate(samples):
        location = f"corpus.asr.samples[{index}]"
        sample = require_object(raw, location)
        require_keys(
            sample, location, {"id", "language", "audio", "durationMs", "sampleRateHz", "coverageTags", "referenceText"}
        )
        sample_id = require_string(sample["id"], f"{location}.id")
        if sample_id in seen:
            raise EvidenceError(f"duplicate ASR sample id: {sample_id}")
        seen.add(sample_id)
        _require_language(sample["language"], f"{location}.language")
        resolve_hashed_file(sample["audio"], base, f"{location}.audio")
        duration = require_number(sample["durationMs"], f"{location}.durationMs")
        sample_rate = require_number(sample["sampleRateHz"], f"{location}.sampleRateHz")
        if duration <= 0 or not sample_rate.is_integer() or sample_rate <= 0:
            raise EvidenceError(f"{location} duration and sample rate must be positive")
        has_ten_minute_sample = has_ten_minute_sample or duration >= 600_000
        sample_rates.add(int(sample_rate))
        tags = {
            require_string(value, f"{location}.coverageTags")
            for value in require_list(sample["coverageTags"], f"{location}.coverageTags")
        }
        coverage_tags.update(tags)
        require_text(sample["referenceText"], f"{location}.referenceText")
    if not ASR_COVERAGE_TAGS.issubset(coverage_tags) or len(sample_rates) < 2 or not has_ten_minute_sample:
        raise EvidenceError(
            "ASR corpus must cover the required content conditions, two sample rates, and a ten-minute recording"
        )


def _validate_alignment_corpus(section: dict[str, Any], base: Path) -> None:
    require_keys(section, "corpus.alignment", {"thresholds", "samples"})
    thresholds = require_object(section["thresholds"], "corpus.alignment.thresholds")
    require_keys(
        thresholds,
        "corpus.alignment.thresholds",
        {"minimumCoverage", "maximumMedianBoundaryErrorMs", "maximumP95BoundaryErrorMs"},
    )
    coverage = require_number(thresholds["minimumCoverage"], "corpus.alignment.thresholds.minimumCoverage")
    median = require_number(
        thresholds["maximumMedianBoundaryErrorMs"], "corpus.alignment.thresholds.maximumMedianBoundaryErrorMs"
    )
    p95 = require_number(
        thresholds["maximumP95BoundaryErrorMs"], "corpus.alignment.thresholds.maximumP95BoundaryErrorMs"
    )
    if not 0.0 <= coverage <= 1.0 or median < 0.0 or p95 < median:
        raise EvidenceError("alignment thresholds are inconsistent")
    samples = require_list(section["samples"], "corpus.alignment.samples")
    _require_languages(samples, "corpus.alignment.samples")
    seen: set[str] = set()
    for index, raw in enumerate(samples):
        location = f"corpus.alignment.samples[{index}]"
        sample = require_object(raw, location)
        require_keys(sample, location, {"id", "language", "audio", "durationMs", "sampleRateHz", "units"})
        sample_id = require_string(sample["id"], f"{location}.id")
        if sample_id in seen:
            raise EvidenceError(f"duplicate alignment sample id: {sample_id}")
        seen.add(sample_id)
        _require_language(sample["language"], f"{location}.language")
        resolve_hashed_file(sample["audio"], base, f"{location}.audio")
        if (
            require_number(sample["durationMs"], f"{location}.durationMs") <= 0
            or require_number(sample["sampleRateHz"], f"{location}.sampleRateHz") <= 0
        ):
            raise EvidenceError(f"{location} duration and sample rate must be positive")
        units = require_list(sample["units"], f"{location}.units")
        if not units:
            raise EvidenceError(f"{location}.units must not be empty")
        unit_ids: set[str] = set()
        for unit_index, raw_unit in enumerate(units):
            unit_location = f"{location}.units[{unit_index}]"
            unit = require_object(raw_unit, unit_location)
            require_keys(unit, unit_location, {"id", "text", "startMs", "endMs"})
            unit_id = require_string(unit["id"], f"{unit_location}.id")
            start = require_number(unit["startMs"], f"{unit_location}.startMs")
            end = require_number(unit["endMs"], f"{unit_location}.endMs")
            if unit_id in unit_ids or start < 0.0 or end < start:
                raise EvidenceError(f"{unit_location} has invalid identity or timing")
            unit_ids.add(unit_id)


def _validate_tts_corpus(section: dict[str, Any]) -> None:
    require_keys(section, "corpus.tts", {"thresholds", "samples"})
    thresholds = require_object(section["thresholds"], "corpus.tts.thresholds")
    require_keys(thresholds, "corpus.tts.thresholds", TTS_CRITERIA)
    for criterion in TTS_CRITERIA:
        score = require_number(thresholds[criterion], f"corpus.tts.thresholds.{criterion}")
        if not 1.0 <= score <= 5.0:
            raise EvidenceError(f"TTS threshold {criterion} must be between 1 and 5")
    samples = require_list(section["samples"], "corpus.tts.samples")
    if len(samples) < 30:
        raise EvidenceError("corpus.tts.samples must contain at least 30 fixed samples")
    _require_languages(samples, "corpus.tts.samples")
    seen: set[str] = set()
    for index, raw in enumerate(samples):
        location = f"corpus.tts.samples[{index}]"
        sample = require_object(raw, location)
        require_keys(sample, location, {"id", "language", "text"})
        sample_id = require_string(sample["id"], f"{location}.id")
        if sample_id in seen:
            raise EvidenceError(f"duplicate TTS sample id: {sample_id}")
        seen.add(sample_id)
        _require_language(sample["language"], f"{location}.language")
        require_string(sample["text"], f"{location}.text")


def _require_languages(samples: list[Any], location: str) -> None:
    languages = {require_object(sample, f"{location}[{index}]").get("language") for index, sample in enumerate(samples)}
    if languages != LANGUAGES:
        raise EvidenceError(f"{location} must cover exactly zh-CN and en-US")


def _require_language(value: object, location: str) -> str:
    language = require_string(value, location)
    if language not in LANGUAGES:
        raise EvidenceError(f"{location} must be zh-CN or en-US")
    return language
