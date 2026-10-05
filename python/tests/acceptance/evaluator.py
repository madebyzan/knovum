from __future__ import annotations

from collections.abc import Callable
from pathlib import Path
from typing import Any, TypeVar

from acceptance.alignment_evaluation import evaluate_alignment
from acceptance.capture_manifest import CaptureManifest, Unavailable
from acceptance.corpus import Corpus
from acceptance.embedding_evaluation import evaluate_embedding
from acceptance.evidence import EvidenceError, resolve_hashed_file, sha256_file
from acceptance.transcription_evaluation import evaluate_transcription
from acceptance.tts_evaluation import evaluate_tts

Section = TypeVar("Section")


def evaluate(corpus: Corpus, capture: CaptureManifest, corpus_path: Path, capture_path: Path) -> dict[str, Any]:
    provenance = capture.provenance
    if provenance.corpus_sha256 != sha256_file(corpus_path):
        raise EvidenceError("capture provenance does not match the supplied corpus bytes")
    for samples in (corpus.asr.samples, corpus.alignment.samples):
        for sample in samples:
            resolve_hashed_file(sample.audio, corpus_path.parent, f"corpus.{sample.id}.audio")
    capabilities = capture.capabilities
    base = capture_path.parent
    evaluations = {
        "embedding": evaluate_or_unavailable(
            capabilities.embedding, lambda section: evaluate_embedding(corpus.embedding, section, base, provenance)
        ),
        "asr": evaluate_or_unavailable(
            capabilities.asr, lambda section: evaluate_transcription(corpus.asr, section, base, provenance)
        ),
        "alignment": evaluate_or_unavailable(
            capabilities.alignment, lambda section: evaluate_alignment(corpus.alignment, section, base, provenance)
        ),
        "tts": evaluate_or_unavailable(
            capabilities.tts, lambda section: evaluate_tts(corpus.tts, section, base, provenance)
        ),
    }
    statuses = {item["status"] for item in evaluations.values()}
    overall = "failed" if "failed" in statuses else "unavailable" if "unavailable" in statuses else "passed"
    return {
        "schemaVersion": 1,
        "status": overall,
        "runId": capture.run_id,
        "capturedAt": capture.captured_at.isoformat(),
        "corpus": {"id": corpus.corpus_id, "version": corpus.corpus_version, "sha256": provenance.corpus_sha256},
        "provenance": provenance.model_dump(mode="json", by_alias=True),
        "capabilities": evaluations,
    }


def evaluate_or_unavailable(
    section: Section | Unavailable, evaluator: Callable[[Section], dict[str, Any]]
) -> dict[str, Any]:
    if isinstance(section, Unavailable):
        return section.model_dump()
    return evaluator(section)
