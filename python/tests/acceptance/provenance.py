"""Bind scored outputs to the preregistered candidate and reference identities."""

from __future__ import annotations

from pathlib import Path
from typing import TypeVar

from acceptance.contracts import Capability, EvidenceModel, HashedFile
from acceptance.evidence import EvidenceError, load_evidence, resolve_hashed_file
from acceptance.execution import ExecutionEvidence, SourceKind
from acceptance.results import Provenance

Values = TypeVar("Values", bound=EvidenceModel)


def load_execution_evidence(
    reference: HashedFile,
    base: Path,
    location: str,
    expected_method: str,
    capability: Capability,
    provenance: Provenance,
    expected_kind: SourceKind,
    model: type[ExecutionEvidence[Values]],
    expected_model_spec: str | None = None,
    expected_config: str | None = None,
) -> Values:
    output = load_evidence(resolve_hashed_file(reference, base, location), model)
    source = output.source
    if source.kind != expected_kind or source.method != expected_method:
        raise EvidenceError(f"{location} is not evidence from the required native execution path")
    if expected_model_spec is not None and source.model_spec_id != expected_model_spec:
        raise EvidenceError(f"{location} model differs from the fixed reference profile")
    if expected_config is not None and source.config_id != expected_config:
        raise EvidenceError(f"{location} config differs from the fixed reference profile")
    if source.kind == "bundled_worker":
        if source.runtime_digest != provenance.runtime_digest:
            raise EvidenceError(f"{location} runtime digest differs from the candidate runtime")
        if source.model_spec_id != provenance.models.select(capability).spec_id:
            raise EvidenceError(f"{location} model differs from the frozen candidate model")
        if source.config_id != provenance.configs.select(capability).id:
            raise EvidenceError(f"{location} config differs from the frozen candidate config")
    return output.values
