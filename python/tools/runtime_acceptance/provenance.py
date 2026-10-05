from __future__ import annotations

from pathlib import Path
from typing import Any

from runtime_acceptance.evidence import (
    EvidenceError,
    require_keys,
    require_number,
    require_object,
    require_sha256,
    require_string,
    require_timestamp,
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
