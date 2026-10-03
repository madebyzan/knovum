from __future__ import annotations

from pathlib import Path
from typing import Any

from knovum_inference.contracts import SpeechDiarizeParams
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech_common import check_deadline, milliseconds, verify_audio


class DiarizationEngine:
    def __init__(self, model_directory: Path, device: str) -> None:
        self._model_directory = model_directory
        self._device = device
        self._pipeline: Any = None

    def diarize(self, params: SpeechDiarizeParams) -> dict[str, Any]:
        audio_path = verify_audio(params.audio_ref)
        check_deadline(params.deadline_ms)
        bounds = params.speaker_count_bounds
        if bounds.minimum == 0 or bounds.maximum == 0:
            raise invalid_params("Speaker count bounds must be positive")
        if bounds.minimum and bounds.maximum and bounds.minimum > bounds.maximum:
            raise invalid_params("Speaker count bounds are reversed")
        if self._pipeline is None:
            try:
                import torch
                from pyannote.audio import Pipeline
            except ImportError as error:
                raise server_error("runtime_mismatch", "pyannote.audio is unavailable") from error
            self._pipeline = Pipeline.from_pretrained(str(self._model_directory))
            self._pipeline.to(torch.device(self._device))
        kwargs: dict[str, int] = {}
        if bounds.minimum is not None:
            kwargs["min_speakers"] = bounds.minimum
        if bounds.maximum is not None:
            kwargs["max_speakers"] = bounds.maximum
        output = self._pipeline(str(audio_path), **kwargs)
        turns = [
            {"speakerId": str(speaker), "startMs": milliseconds(turn.start), "endMs": milliseconds(turn.end)}
            for turn, _, speaker in output.speaker_diarization.itertracks(yield_label=True)
        ]
        check_deadline(params.deadline_ms)
        return {
            "requestId": params.request_id,
            "attemptEpoch": params.attempt_epoch,
            "modelSpecId": params.model_spec_id,
            "turns": turns,
        }
