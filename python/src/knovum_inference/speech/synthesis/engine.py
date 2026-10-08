from __future__ import annotations

import threading
from pathlib import Path
from typing import Any

from knovum_inference.contracts.generated.synthesis import SpeechSynthesisEngine, SpeechSynthesizeParams
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech.request import check_deadline
from knovum_inference.speech.synthesis.moss_nano import MossNanoSynthesisEngine
from knovum_inference.speech.synthesis.stream import SynthesisChunkPublisher
from knovum_inference.speech.synthesis.voxcpm2 import (
    FULL_MODEL_SPEC_ID,
    Q8_MODEL_SPEC_ID,
    VoxCpm2SynthesisEngine,
)


class SynthesisEngine:
    def __init__(
        self,
        model_directory: Path,
        device: str,
        cpu_threads: int,
        model_spec_id: str,
    ) -> None:
        if model_spec_id not in {
            "moss-tts-nano-onnx-v1",
            FULL_MODEL_SPEC_ID,
            Q8_MODEL_SPEC_ID,
        }:
            raise server_error("runtime_mismatch", "Synthesis model specification is unsupported")
        self._moss_nano = (
            MossNanoSynthesisEngine(model_directory, device, cpu_threads)
            if model_spec_id == "moss-tts-nano-onnx-v1"
            else None
        )
        self._voxcpm2 = (
            VoxCpm2SynthesisEngine(model_directory, device, model_spec_id, cpu_threads)
            if model_spec_id in {FULL_MODEL_SPEC_ID, Q8_MODEL_SPEC_ID}
            else None
        )

    def synthesize(
        self,
        params: SpeechSynthesizeParams,
        publisher: SynthesisChunkPublisher,
        cancelled: threading.Event,
    ) -> dict[str, Any]:
        if not params.text.strip() or len(params.text) > 10_000:
            raise invalid_params("Synthesis text is invalid")
        check_deadline(params.deadline_ms)
        if params.profile.engine is SpeechSynthesisEngine.vox_cpm2:
            if self._voxcpm2 is None:
                raise invalid_params("VoxCPM2 synthesis requires a VoxCPM2 model")
            self._voxcpm2.generate(params, publisher, cancelled)
        elif params.profile.engine is SpeechSynthesisEngine.moss_nano:
            if self._moss_nano is None:
                raise invalid_params("MOSS synthesis requires the MOSS model")
            self._moss_nano.generate(params, publisher, cancelled)
        else:
            raise invalid_params("Synthesis engine is unsupported")
        return {"modelSpecId": params.model_spec_id, **publisher.finish()}
