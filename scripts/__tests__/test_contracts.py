from __future__ import annotations

import importlib
import json
from pathlib import Path

from pydantic import BaseModel

from generate_python_contracts import CONTRACT_MODULES


def test_modular_contracts_cover_rust_definitions_without_aliases_or_duplicate_types() -> None:
    source = Path(__file__).resolve().parents[2] / "python/schemas/inference.schema.json"
    definitions = json.loads(source.read_text())["$defs"]
    generated: dict[str, type] = {}
    for owner, names in CONTRACT_MODULES.items():
        module = importlib.import_module(f"knovum_inference.contracts.generated.{owner}")
        for name in names:
            model = getattr(module, name)
            assert name not in generated
            assert model.__name__ == name
            assert model.__module__ == module.__name__
            generated[name] = model
            definition = definitions[name]
            if issubclass(model, BaseModel):
                assert {field.alias or name for name, field in model.model_fields.items()} == definition[
                    "properties"
                ].keys()
                assert model.model_config["extra"] == "forbid"
                assert model.model_config["strict"] is True
                assert {
                    field.alias or name for name, field in model.model_fields.items() if field.is_required()
                } == set(definition.get("required", []))
    assert generated.keys() == definitions.keys()


def test_model_resource_contract_matches_rust_request_and_provider_inventory() -> None:
    from knovum_model_resources.contracts import DownloadRequest, HuggingFaceSource, ModelScopeSource, SnapshotFile

    source = Path(__file__).resolve().parents[2] / "python/schemas/model-resources.schema.json"
    schema = json.loads(source.read_text())
    sources = {variant["title"]: variant for variant in schema["$defs"]["HubSource"]["oneOf"]}
    for model, definition in [
        (DownloadRequest, schema),
        (SnapshotFile, schema["$defs"]["SnapshotFile"]),
        (HuggingFaceSource, sources["HuggingFaceSource"]),
        (ModelScopeSource, sources["ModelScopeSource"]),
    ]:
        assert model.model_fields.keys() == definition["properties"].keys()
        assert model.model_config["strict"] is True
        assert model.model_config["extra"] == "forbid"
        assert {name for name, field in model.model_fields.items() if field.is_required()} == set(
            definition["required"]
        )
