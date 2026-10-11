from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path
from typing import Any

# This map selects file ownership only. Rust remains the source of every wire field and type name.
AGENT_RUNTIME_CONTRACT_MODULES = {
    "control": {
        "AgentInitialize",
        "AgentReady",
        "AgentExecutionIdentity",
        "AgentExecute",
        "AgentSkillFile",
        "SkillFileEncoding",
        "AgentControl",
        "AgentControlOperation",
        "AgentPruneThreads",
        "AgentExecutionObservation",
        "AgentExecutionState",
    },
    "messages": {
        "AgentContentPart",
        "AgentInputMessage",
        "AgentInputRole",
        "AgentProjectionRequest",
        "AgentProjectedMessage",
        "AgentProjection",
        "AgentContentDelta",
        "AgentPlanItem",
        "AgentPlanState",
    },
    "models": {
        "ChatModelProtocol",
        "AgentModelBinding",
        "AgentModelPurpose",
        "AgentModelAdmission",
        "AgentModelCompletion",
        "AgentModelFailure",
        "AgentModelCheckpoint",
    },
    "tools": {
        "AgentToolDefinition",
        "AgentToolInvocation",
        "AgentToolOutcome",
        "AgentToolAdmission",
    },
}


def codegen_options(base_class: str) -> list[str]:
    return [
        "--output-model-type",
        "pydantic_v2.BaseModel",
        "--target-python-version",
        "3.11",
        "--disable-timestamp",
        "--base-class",
        base_class,
        "--use-annotated",
        "--use-union-operator",
        "--use-standard-collections",
        "--use-double-quotes",
        "--use-title-as-name",
        "--formatters",
        "black",
        "isort",
    ]


def namespace_references(value: Any, owners: dict[str, str]) -> Any:
    if isinstance(value, list):
        return [namespace_references(item, owners) for item in value]
    if not isinstance(value, dict):
        return value
    return {
        key: f"#/$defs/{owners[item[8:]]}.{item[8:]}"
        if key == "$ref" and item.startswith("#/$defs/")
        else namespace_references(item, owners)
        for key, item in value.items()
    }


def generate_owned_contracts(capability: str, modules: dict[str, set[str]], schema_name: str) -> None:
    root = Path(__file__).resolve().parents[1] / "python"
    schema = json.loads((root / f"schemas/{capability}.schema.json").read_text())
    owners = {name: module for module, names in modules.items() for name in names}
    if len(owners) != sum(len(names) for names in modules.values()) or owners.keys() != schema["$defs"].keys():
        raise ValueError("Every Rust wire definition must have exactly one Python contract module")
    schema = namespace_references(schema, owners)
    schema["$defs"] = {f"{owners[name]}.{name}": definition for name, definition in schema["$defs"].items()}
    for name, definition in schema["$defs"].items():
        definition["title"] = name
        for position, variant in enumerate(definition.get("oneOf", []), start=1):
            variant["title"] = f"{name}Variant{position}"
    schema["title"] = f"schema.{schema_name}"
    output = root / f"src/knovum_{capability}/contracts/generated"
    with tempfile.TemporaryDirectory(prefix="knovum-contracts-") as directory:
        source = Path(directory) / f"{capability}.schema.json"
        source.write_text(json.dumps(schema))
        generated = Path(directory) / "generated"
        subprocess.run(
            [
                "datamodel-codegen",
                "--input",
                str(source),
                "--input-file-type",
                "jsonschema",
                "--output",
                str(generated),
                *codegen_options(f"knovum_{capability}.contracts.wire_model.WireModel"),
                "--snake-case-field",
                "--use-default",
                "--treat-dot-as-module",
            ],
            check=True,
        )
        output.mkdir(parents=True, exist_ok=True)
        for stale in output.glob("*.py"):
            if not (generated / stale.name).is_file():
                stale.unlink()
        for module in generated.glob("*.py"):
            (output / module.name).write_text(module.read_text())


def main() -> None:
    generate_owned_contracts("agent_runtime", AGENT_RUNTIME_CONTRACT_MODULES, "AgentWireSchemas")


if __name__ == "__main__":
    main()
