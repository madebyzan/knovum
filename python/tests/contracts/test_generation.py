from __future__ import annotations

import importlib
import json
from pathlib import Path
from typing import get_args

from pydantic import BaseModel, RootModel

from generate_python_contracts import AGENT_RUNTIME_CONTRACT_MODULES


def test_modular_contracts_cover_rust_definitions_without_aliases_or_duplicate_types() -> None:
    source = Path(__file__).resolve().parents[2] / "schemas/agent_runtime.schema.json"
    definitions = json.loads(source.read_text())["$defs"]
    generated: dict[str, type] = {}
    for owner, names in AGENT_RUNTIME_CONTRACT_MODULES.items():
        module = importlib.import_module(f"knovum_agent_runtime.contracts.generated.{owner}")
        for name in names:
            model = getattr(module, name)
            assert name not in generated
            assert model.__name__ == name
            assert model.__module__.startswith("knovum_agent_runtime.contracts.generated.")
            generated[name] = model
            definition = definitions[name]
            if issubclass(model, BaseModel):
                model.model_rebuild()
            if issubclass(model, RootModel):
                variants = get_args(model.model_fields["root"].annotation)
                assert len(variants) == len(definition["oneOf"])
                for variant, variant_schema in zip(variants, definition["oneOf"], strict=True):
                    assert {field.alias or key for key, field in variant.model_fields.items()} == variant_schema[
                        "properties"
                    ].keys()
                    assert variant.model_config["extra"] == "forbid"
                    assert variant.model_config["strict"] is True
            elif issubclass(model, BaseModel):
                assert {field.alias or name for name, field in model.model_fields.items()} == definition[
                    "properties"
                ].keys()
                assert model.model_config["extra"] == "forbid"
                assert model.model_config["strict"] is True
                assert {
                    field.alias or name for name, field in model.model_fields.items() if field.is_required()
                } == set(definition.get("required", []))
    assert generated.keys() == definitions.keys()
