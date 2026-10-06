from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path

# This map selects file ownership only. Rust remains the source of every wire field and type name.
CONTRACT_MODULES = {
    "diagnostics": {
        "RuntimeDiagnostic",
        "RuntimeDiagnosticFrame",
        "RuntimeDiagnosticEvent",
        "RuntimeDiagnosticMethod",
        "RuntimeDiagnosticCode",
    },
    "control": {
        "RuntimeCapability",
        "RuntimeDeviceProfile",
        "RuntimeModelPaths",
        "RuntimeLimits",
        "RuntimeHealthParams",
        "RuntimeHealthObservation",
        "RuntimeCancelParams",
        "RuntimeCancellationAcknowledgement",
        "RuntimeAckChunkParams",
        "RuntimeShutdownParams",
        "RuntimeShutdownAcknowledgement",
        "RuntimeInitializeParams",
        "RuntimeSession",
    },
    "embedding": {"EmbeddingRole", "EmbeddingEncoding", "EmbeddingEncodeParams"},
    "image": {"TensorDataType", "TensorFileReference", "ImageSegmentation", "ImageSegmentParams"},
    "audio": {"AudioFileReference"},
    "transcription": {
        "AsrEngine",
        "LanguagePolicy",
        "SpeechFileReference",
        "SpeechWordTiming",
        "SpeechProgressEventType",
        "AsrProfile",
        "SpeechTranscription",
        "SpeechTranscriptSegment",
        "SpeechProgressEvent",
        "SpeechTranscribeParams",
        "SpeechTranscriptArtifact",
    },
    "alignment": {
        "AlignmentTextSegment",
        "AlignmentProfile",
        "AlignmentUnitKind",
        "SpeechAlignParams",
        "AlignedSpeechUnit",
        "SpeechAlignment",
    },
    "diarization": {"SpeakerCountBounds", "SpeakerTurn", "SpeechDiarizeParams", "SpeechDiarization"},
    "synthesis": {
        "VoiceSourceKind",
        "SpeechSynthesisEngine",
        "SpeechSynthesisChunkGranularity",
        "SynthesizedAudioFileReference",
        "SpeechSynthesis",
        "VoiceRevisionReference",
        "SynthesisProfile",
        "SpeechSynthesisChunkEvent",
        "SpeechSynthesizeParams",
    },
    "separation": {"SpeechSeparateParams", "SpeechSeparation"},
}

ASSISTANT_CONTRACT_MODULES = {
    "control": {
        "AssistantInitialize",
        "AssistantReady",
        "AssistantExecutionIdentity",
        "AssistantExecute",
        "AssistantControl",
        "AssistantControlOperation",
        "AssistantPruneThreads",
        "AssistantExecutionResult",
        "AssistantExecutionState",
    },
    "messages": {
        "AssistantContentPart",
        "AssistantInputMessage",
        "AssistantInputRole",
        "AssistantProjectionRequest",
        "AssistantProjectedMessage",
        "AssistantProjection",
        "AssistantContentDelta",
        "AssistantPlanItem",
        "AssistantPlanState",
    },
    "models": {
        "ChatModelProtocol",
        "AssistantModelBinding",
        "AssistantModelPurpose",
        "AssistantModelAdmission",
        "AssistantModelCompletion",
        "AssistantModelFailure",
        "AssistantModelCheckpoint",
    },
    "tools": {
        "AssistantToolDefinition",
        "AssistantToolInvocation",
        "AssistantToolOutcome",
        "AssistantToolAdmission",
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


def generate_owned_contracts(capability: str, modules: dict[str, set[str]], schema_name: str) -> None:
    root = Path(__file__).resolve().parents[1] / "python"
    schema = json.loads((root / f"schemas/{capability}.schema.json").read_text())
    owners = {name: module for module, names in modules.items() for name in names}
    if len(owners) != sum(len(names) for names in modules.values()) or owners.keys() != schema["$defs"].keys():
        raise ValueError("Every Rust wire definition must have exactly one Python contract module")
    for name, definition in schema["$defs"].items():
        definition["title"] = f"{owners[name]}.{name}"
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


def generate_model_resources() -> None:
    root = Path(__file__).resolve().parents[1] / "python"
    subprocess.run(
        [
            "datamodel-codegen",
            "--input",
            str(root / "schemas/model-resources.schema.json"),
            "--input-file-type",
            "jsonschema",
            "--output",
            str(root / "src/knovum_model_resources/contracts.py"),
            *codegen_options("knovum_model_resources.wire_model.WireModel"),
            "--collapse-root-models",
        ],
        check=True,
    )


def main() -> None:
    generate_owned_contracts("inference", CONTRACT_MODULES, "InferenceWireSchemas")
    generate_owned_contracts("assistant", ASSISTANT_CONTRACT_MODULES, "AssistantWireSchemas")
    generate_model_resources()


if __name__ == "__main__":
    main()
