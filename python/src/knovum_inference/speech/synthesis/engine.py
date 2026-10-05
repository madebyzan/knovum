from __future__ import annotations

import threading
from pathlib import Path
from typing import Any

from knovum_inference.contracts.generated.synthesis import SpeechSynthesisEngine, SpeechSynthesizeParams
from knovum_inference.errors import invalid_params
from knovum_inference.speech.request import check_deadline
from knovum_inference.speech.synthesis.kokoro import KokoroSynthesisEngine
from knovum_inference.speech.synthesis.stream import SynthesisChunkPublisher
from knovum_inference.speech.synthesis.voxcpm2 import VoxCpm2SynthesisEngine


class SynthesisEngine:
    def __init__(self, model_directory: Path, device: str, cpu_threads: int) -> None:
        self._kokoro = KokoroSynthesisEngine(model_directory, device, cpu_threads)
        self._voxcpm2 = VoxCpm2SynthesisEngine(model_directory, device)

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
            self._voxcpm2.generate(params, publisher, cancelled)
        elif params.profile.engine is SpeechSynthesisEngine.kokoro:
            self._kokoro.generate(params, publisher, cancelled)
        else:
            raise invalid_params("Synthesis engine is unsupported")
        return {"modelSpecId": params.model_spec_id, **publisher.finish()}
