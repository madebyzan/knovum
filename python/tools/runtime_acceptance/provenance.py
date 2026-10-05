from __future__ import annotations

from pathlib import Path
from typing import Any

from runtime_acceptance.evidence import (
    EvidenceError,
    load_json,
    require_keys,
    require_list,
    require_number,
    require_object,
    require_sha256,
    require_string,
    require_timestamp,
    resolve_hashed_file,
    sha256_file,
)
from runtime_acceptance.schema import COMMIT_PATTERN, TARGETS


def validate_provenance(results: dict[str, Any], corpus_path: Path) -> dict[str, Any]:
    require_keys(results, "results", {"schemaVersion", "runId", "capturedAt", "provenance", "capabilities"})
    if results["schemaVersion"] != 1:
        raise EvidenceError("results.schemaVersion must be 1")
    require_string(results["runId"], "results.runId")
    require_timestamp(results["capturedAt"], "results.capturedAt")
    provenance = require_object(results["provenance"], "results.provenance")
    require_keys(
        provenance,
        "results.provenance",
        {
            "appVersion",
            "appCommit",
            "runtimeVersion",
            "runtimeDigest",
            "target",
            "models",
            "configs",
            "corpusSha256",
            "device",
        },
    )
    require_string(provenance["appVersion"], "results.provenance.appVersion")
    if not COMMIT_PATTERN.fullmatch(require_string(provenance["appCommit"], "results.provenance.appCommit")):
        raise EvidenceError("results.provenance.appCommit must be a lowercase full Git commit")
    require_string(provenance["runtimeVersion"], "results.provenance.runtimeVersion")
    require_sha256(provenance["runtimeDigest"], "results.provenance.runtimeDigest")
    target = require_string(provenance["target"], "results.provenance.target")
    if target not in TARGETS:
        raise EvidenceError("results.provenance.target is outside the approved release matrix")
    models = require_object(provenance["models"], "results.provenance.models")
    configs = require_object(provenance["configs"], "results.provenance.configs")
    required_capabilities = {"embedding", "asr", "alignment", "tts"}
    if models.keys() != required_capabilities or configs.keys() != required_capabilities:
        raise EvidenceError("provenance models and configs must identify all acceptance capabilities")
    for capability in required_capabilities:
        model = require_object(models[capability], f"results.provenance.models.{capability}")
        require_keys(model, f"results.provenance.models.{capability}", {"specId", "identityDigest"})
        require_string(model["specId"], f"results.provenance.models.{capability}.specId")
        require_sha256(model["identityDigest"], f"results.provenance.models.{capability}.identityDigest")
        config = require_object(configs[capability], f"results.provenance.configs.{capability}")
        require_keys(config, f"results.provenance.configs.{capability}", {"id", "parameters"})
        require_string(config["id"], f"results.provenance.configs.{capability}.id")
        require_object(config["parameters"], f"results.provenance.configs.{capability}.parameters")
    corpus_digest = require_sha256(provenance["corpusSha256"], "results.provenance.corpusSha256")
    if corpus_digest != sha256_file(corpus_path):
        raise EvidenceError("results provenance does not match the supplied corpus bytes")
    device = require_object(provenance["device"], "results.provenance.device")
    require_keys(
        device,
        "results.provenance.device",
        {
            "manufacturer",
            "model",
            "cpu",
            "physicalCores",
            "memoryBytes",
            "os",
            "osVersion",
            "powerMode",
            "accelerator",
            "driver",
        },
    )
    for key in {"manufacturer", "model", "cpu", "os", "osVersion", "powerMode", "accelerator"}:
        require_string(device[key], f"results.provenance.device.{key}")
    if device["driver"] is not None:
        require_string(device["driver"], "results.provenance.device.driver")
    if (
        require_number(device["physicalCores"], "results.provenance.device.physicalCores") <= 0
        or require_number(device["memoryBytes"], "results.provenance.device.memoryBytes") <= 0
    ):
        raise EvidenceError("device core and memory values must be positive")
    return provenance


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
    if source["runtimeDigest"] != provenance["runtimeDigest"] and source_kind == "bundled_worker":
        raise EvidenceError(f"{location} runtime digest differs from the candidate runtime")
    require_sha256(source["runtimeDigest"], f"{location}.source.runtimeDigest")
    for key in {"modelSpecId", "configId"}:
        require_string(source[key], f"{location}.source.{key}")
    if expected_model_spec is not None and source["modelSpecId"] != expected_model_spec:
        raise EvidenceError(f"{location} model differs from the fixed reference profile")
    if expected_config is not None and source["configId"] != expected_config:
        raise EvidenceError(f"{location} config differs from the fixed reference profile")
    if source_kind == "bundled_worker":
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
