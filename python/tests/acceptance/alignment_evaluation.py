from __future__ import annotations

from pathlib import Path
from typing import Any

import numpy as np

from acceptance.corpus import AlignmentCorpus, AlignmentThresholds
from acceptance.evidence import EvidenceError
from acceptance.execution import AlignmentValues, ExecutionEvidence
from acceptance.provenance import load_execution_evidence
from acceptance.results import AlignmentResult, Provenance


def evaluate_alignment(
    corpus: AlignmentCorpus, section: AlignmentResult, base: Path, provenance: Provenance
) -> dict[str, Any]:
    values = load_execution_evidence(
        section.candidate_output,
        base,
        "alignment.candidateOutput",
        "speech.align",
        "alignment",
        provenance,
        "bundled_worker",
        ExecutionEvidence[AlignmentValues],
    )
    if values.samples.keys() != {item.id for item in corpus.samples}:
        raise EvidenceError("alignment output identities differ from the fixed corpus")
    by_language: dict[str, dict[str, float | None]] = {}
    all_errors: list[float] = []
    aligned_total = 0
    unit_total = 0
    for language in ("zh-CN", "en-US"):
        errors: list[float] = []
        aligned = 0
        total = 0
        for sample in corpus.samples:
            if sample.language != language:
                continue
            expected_units = {unit.id: unit for unit in sample.units}
            returned = values.samples[sample.id]
            actual_units = {unit.id: unit for unit in returned}
            if len(actual_units) != len(returned) or actual_units.keys() != expected_units.keys():
                raise EvidenceError(f"alignment units for {sample.id} differ from the fixed annotation")
            total += len(expected_units)
            for unit_id, expected in expected_units.items():
                actual = actual_units[unit_id]
                if actual.start_ms is not None and actual.end_ms is not None:
                    aligned += 1
                    errors.extend((abs(actual.start_ms - expected.start_ms), abs(actual.end_ms - expected.end_ms)))
        by_language[language] = alignment_metrics(aligned, total, errors)
        aligned_total += aligned
        unit_total += total
        all_errors.extend(errors)
    overall = alignment_metrics(aligned_total, unit_total, all_errors)
    passed = all(_alignment_passes(item, corpus.thresholds) for item in [overall, *by_language.values()])
    return {
        "status": "passed" if passed else "failed",
        "thresholds": corpus.thresholds.model_dump(by_alias=True),
        "overall": overall,
        "byLanguage": by_language,
    }


def alignment_metrics(aligned: int, total: int, errors: list[float]) -> dict[str, float | None]:
    if total == 0:
        raise EvidenceError("alignment corpus must contain annotated units for each language")
    percentiles = np.percentile(errors, [50, 95], method="linear") if errors else None
    return {
        "coverage": aligned / total,
        "medianBoundaryErrorMs": float(percentiles[0]) if percentiles is not None else None,
        "p95BoundaryErrorMs": float(percentiles[1]) if percentiles is not None else None,
    }


def _alignment_passes(metrics: dict[str, float | None], thresholds: AlignmentThresholds) -> bool:
    median = metrics["medianBoundaryErrorMs"]
    p95 = metrics["p95BoundaryErrorMs"]
    coverage = metrics["coverage"]
    return (
        median is not None
        and p95 is not None
        and coverage is not None
        and coverage >= thresholds.minimum_coverage
        and median <= thresholds.maximum_median_boundary_error_ms
        and p95 <= thresholds.maximum_p95_boundary_error_ms
    )
