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


def main() -> None:
    root = Path(__file__).resolve().parents[1] / "python"
    schema = json.loads((root / "schemas/runtime.schema.json").read_text())
    owners = {name: module for module, names in CONTRACT_MODULES.items() for name in names}
    if len(owners) != sum(len(names) for names in CONTRACT_MODULES.values()) or owners.keys() != schema["$defs"].keys():
        raise ValueError("Every Rust wire definition must have exactly one Python contract module")
    for name, definition in schema["$defs"].items():
        definition["title"] = f"{owners[name]}.{name}"
    schema["title"] = "schema.InferenceWireSchemas"
    output = root / "src/knovum_inference/contracts/generated"
    with tempfile.TemporaryDirectory(prefix="knovum-contracts-") as directory:
        source = Path(directory) / "runtime.schema.json"
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
                "--output-model-type",
                "pydantic_v2.BaseModel",
                "--target-python-version",
                "3.11",
                "--disable-timestamp",
                "--base-class",
                "knovum_inference.contracts.wire_model.WireModel",
                "--snake-case-field",
                "--use-annotated",
                "--use-union-operator",
                "--use-standard-collections",
                "--use-default",
                "--use-double-quotes",
                "--use-title-as-name",
                "--treat-dot-as-module",
                "--formatters",
                "black",
                "isort",
            ],
            check=True,
        )
        output.mkdir(parents=True, exist_ok=True)
        for stale in output.glob("*.py"):
            if not (generated / stale.name).is_file():
                stale.unlink()
        for module in generated.glob("*.py"):
            (output / module.name).write_text(module.read_text())


if __name__ == "__main__":
    main()
