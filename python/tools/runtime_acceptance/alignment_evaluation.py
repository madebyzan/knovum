from __future__ import annotations

from collections.abc import Iterable
from pathlib import Path
from typing import Any

from runtime_acceptance.common import (
    EvidenceError,
    percentile,
    require_keys,
    require_list,
    require_number,
    require_object,
    require_string,
)
from runtime_acceptance.execution_evidence import load_execution_evidence


def evaluate_alignment(
    corpus: object, section: dict[str, Any], base: Path, provenance: dict[str, Any]
) -> dict[str, Any]:
    location = "results.capabilities.alignment"
    require_keys(section, location, {"status", "candidateOutput"})
    values = load_execution_evidence(
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
