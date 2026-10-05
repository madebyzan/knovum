from __future__ import annotations

from pathlib import Path
from typing import Any

from runtime_acceptance.common import (
    EvidenceError,
    require_keys,
    require_list,
    require_object,
    require_sha256,
    require_string,
    require_timestamp,
    resolve_hashed_file,
)


def load_execution_evidence(
    reference: object,
    base: Path,
    location: str,
    expected_method: str,
    capability: str,
    provenance: dict[str, Any],
    allowed_kinds: set[str],
    expected_model_spec: str | None = None,
    expected_config: str | None = None,
) -> dict[str, Any]:
    output_path = resolve_hashed_file(reference, base, location)
    from runtime_acceptance.common import load_json

    output = load_json(output_path)
    require_keys(output, location, {"schemaVersion", "source", "values"})
    if output["schemaVersion"] != 1:
        raise EvidenceError(f"{location} schemaVersion must be 1")
    source = require_object(output["source"], f"{location}.source")
    require_keys(
        source, f"{location}.source", {"kind", "method", "runtimeDigest", "modelSpecId", "configId", "invocations"}
    )
    source_kind = require_string(source["kind"], f"{location}.source.kind")
    method = require_string(source["method"], f"{location}.source.method")
    if source_kind not in allowed_kinds or method != expected_method:
        raise EvidenceError(f"{location} is not evidence from the required native execution path")
    if source["runtimeDigest"] != provenance["runtimeDigest"] and source_kind == "native_worker":
        raise EvidenceError(f"{location} runtime digest differs from the candidate runtime")
    require_sha256(source["runtimeDigest"], f"{location}.source.runtimeDigest")
    for key in {"modelSpecId", "configId"}:
        require_string(source[key], f"{location}.source.{key}")
    if expected_model_spec is not None and source["modelSpecId"] != expected_model_spec:
        raise EvidenceError(f"{location} model differs from the fixed reference profile")
    if expected_config is not None and source["configId"] != expected_config:
        raise EvidenceError(f"{location} config differs from the fixed reference profile")
    if source_kind == "native_worker":
        if source["modelSpecId"] != provenance["models"][capability]["specId"]:
            raise EvidenceError(f"{location} model differs from the frozen candidate model")
        if source["configId"] != provenance["configs"][capability]["id"]:
            raise EvidenceError(f"{location} config differs from the frozen candidate config")
    invocations = require_list(source["invocations"], f"{location}.source.invocations")
    if not invocations:
        raise EvidenceError(f"{location} must retain native invocation identities")
    request_ids: set[str] = set()
    for index, raw in enumerate(invocations):
        invocation_location = f"{location}.source.invocations[{index}]"
        invocation = require_object(raw, invocation_location)
        require_keys(invocation, invocation_location, {"requestId", "attemptEpoch", "startedAt", "finishedAt"})
        request_id = require_string(invocation["requestId"], f"{invocation_location}.requestId")
        if request_id in request_ids:
            raise EvidenceError(f"{location} repeats native request id {request_id}")
        request_ids.add(request_id)
        if not isinstance(invocation["attemptEpoch"], int) or invocation["attemptEpoch"] < 0:
            raise EvidenceError(f"{invocation_location}.attemptEpoch must be a non-negative integer")
        started = require_timestamp(invocation["startedAt"], f"{invocation_location}.startedAt")
        finished = require_timestamp(invocation["finishedAt"], f"{invocation_location}.finishedAt")
        if finished < started:
            raise EvidenceError(f"{invocation_location} finishes before it starts")
    return require_object(output["values"], f"{location}.values")
