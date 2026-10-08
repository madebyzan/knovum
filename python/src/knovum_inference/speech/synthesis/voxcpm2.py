from __future__ import annotations

import threading
from pathlib import Path

import numpy as np
from numpy.typing import NDArray

from knovum_inference.contracts.generated.synthesis import SpeechSynthesizeParams, VoiceSourceKind
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech.request import check_deadline
from knovum_inference.speech.synthesis.compiled_voice import CompiledVoice
from knovum_inference.speech.synthesis.reference_audio import read_reference
from knovum_inference.speech.synthesis.stream import SynthesisChunkPublisher

SAMPLE_RATE = 48_000
FULL_MODEL_SPEC_ID = "voxcpm2-2b-gguf-orig-v1"
Q8_MODEL_SPEC_ID = "voxcpm2-2b-gguf-q8_0-v1"
_MODEL_FILES = {
    FULL_MODEL_SPEC_ID: "voxcpm2-orig.gguf",
    Q8_MODEL_SPEC_ID: "voxcpm2-q8_0.gguf",
}


class VoxCpm2SynthesisEngine:
    def __init__(
        self,
        model_directory: Path,
        device: str,
        model_spec_id: str,
        cpu_threads: int = 1,
    ) -> None:
        if model_spec_id not in _MODEL_FILES:
            raise server_error("runtime_mismatch", "VoxCPM2 model specification is unsupported")
        self._model_directory = model_directory
        self._device = device
        self._model_spec_id = model_spec_id
        self._cpu_threads = cpu_threads
        self._engine: CompiledVoice | None = None

    def generate(
        self, params: SpeechSynthesizeParams, publisher: SynthesisChunkPublisher, cancelled: threading.Event
    ) -> None:
        if params.model_spec_id != self._model_spec_id:
            raise invalid_params("VoxCPM2 request differs from the initialized model")
        if params.profile.sample_rate != SAMPLE_RATE or params.profile.speed != 1.0:
            raise invalid_params("VoxCPM2 requires the fixed 48 kHz profile at normal speed")
        if cancelled.is_set():
            raise server_error("cancelled", "Synthesis was cancelled")
        check_deadline(params.deadline_ms)
        text, reference, transcript = self._generation_inputs(params)
        self._load().generate(params, text, reference, transcript, publisher, cancelled)
        if cancelled.is_set():
            raise server_error("cancelled", "Synthesis was cancelled")
        check_deadline(params.deadline_ms)

    def _generation_inputs(self, params: SpeechSynthesizeParams) -> tuple[str, NDArray[np.float32] | None, str | None]:
        revision = params.voice_revision
        if revision.speaker_id is not None:
            raise invalid_params("VoxCPM2 does not accept a preset speaker ID")
        if revision.source_kind is VoiceSourceKind.designed:
            if (
                revision.reference_audio is not None
                or revision.reference_text is not None
                or not revision.design_prompt
                or not revision.design_prompt.strip()
                or params.style is not None
            ):
                raise invalid_params("VoxCPM2 voice design controls are invalid")
            return f"({revision.design_prompt.strip()}){params.text}", None, None
        if revision.source_kind is not VoiceSourceKind.reference or revision.reference_audio is None:
            raise invalid_params("VoxCPM2 cloning requires an immutable reference audio revision")
        reference = read_reference(revision.reference_audio)
        if revision.reference_text is not None:
            if not revision.reference_text.strip() or params.style is not None or revision.design_prompt is not None:
                raise invalid_params("Exact-transcript cloning cannot include style guidance")
            return params.text, reference, revision.reference_text
        controls = [value.strip() for value in (revision.design_prompt, params.style) if value and value.strip()]
        text = f"({', '.join(controls)}){params.text}" if controls else params.text
        return text, reference, None

    def _load(self) -> CompiledVoice:
        if self._engine is None:
            model_file = _MODEL_FILES[self._model_spec_id]
            required = (model_file, "LICENSE", "README.md")
            if not all((self._model_directory / name).is_file() for name in required):
                raise server_error("model_corrupt", "VoxCPM2 model snapshot is incomplete")
            self._engine = CompiledVoice(
                self._model_directory / model_file,
                self._device,
                self._cpu_threads,
            )
        return self._engine
