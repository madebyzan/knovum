from __future__ import annotations

import wave
from pathlib import Path
from typing import Any

from knovum_inference.contracts.generated.diarization import SpeechDiarizeParams
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech.request import check_deadline, milliseconds, verify_audio


class DiarizationEngine:
    def __init__(self, model_directory: Path, device: str) -> None:
        self._model_directory = model_directory
        self._device = device
        self._pipeline: Any = None

    def diarize(self, params: SpeechDiarizeParams) -> dict[str, Any]:
        if self._device != "cpu":
            raise server_error("unsupported_device", "Community-1 diarization requires CPU")
        audio_path = verify_audio(params.audio_ref)
        check_deadline(params.deadline_ms)
        audio = params.audio_ref
        if (audio.media_type, audio.sample_rate, audio.channels, audio.sample_format) != (
            "audio/wav",
            16000,
            1,
            "pcm_s16le",
        ):
            raise invalid_params("Diarization requires mono 16 kHz PCM16 WAV")
        bounds = params.speaker_count_bounds
        if (bounds.minimum is not None and not 1 <= bounds.minimum <= 32) or (
            bounds.maximum is not None and not 1 <= bounds.maximum <= 32
        ):
            raise invalid_params("Speaker count bounds must be within one and 32")
        if bounds.minimum and bounds.maximum and bounds.minimum > bounds.maximum:
            raise invalid_params("Speaker count bounds are reversed")
        pcm = _read_pcm(audio_path)
        if self._pipeline is None:
            required = (
                "config.yaml",
                "embedding/pytorch_model.bin",
                "segmentation/pytorch_model.bin",
                "plda/plda.npz",
                "plda/xvec_transform.npz",
            )
            if not all((self._model_directory / path).is_file() for path in required):
                raise server_error("model_corrupt", "Community-1 model directory is incomplete")
        try:
            import torch

            if self._pipeline is None:
                from pyannote.audio import Pipeline
        except ImportError as error:
            raise server_error("runtime_mismatch", "pyannote.audio is unavailable") from error
        if self._pipeline is None:
            self._pipeline = Pipeline.from_pretrained(str(self._model_directory / "config.yaml"))
            self._pipeline.to(torch.device(self._device))
        kwargs: dict[str, int] = {}
        if bounds.minimum is not None:
            kwargs["min_speakers"] = bounds.minimum
        if bounds.maximum is not None:
            kwargs["max_speakers"] = bounds.maximum
        waveform = torch.frombuffer(pcm, dtype=torch.int16).to(dtype=torch.float32).div_(32768).unsqueeze(0)
        del pcm
        check_deadline(params.deadline_ms)
        output = self._pipeline({"waveform": waveform, "sample_rate": 16000, "uri": audio_path.stem}, **kwargs)
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


def _read_pcm(path: Path) -> bytearray:
    try:
        with wave.open(str(path), "rb") as source:
            frames = source.getnframes()
            if (source.getnchannels(), source.getsampwidth(), source.getframerate(), source.getcomptype()) != (
                1,
                2,
                16000,
                "NONE",
            ) or not 0 < frames <= 2 * 60 * 60 * 16000:
                raise invalid_params("Diarization input must be mono 16 kHz PCM16 and at most two hours")
            pcm = bytearray(source.readframes(frames))
    except (wave.Error, EOFError) as error:
        raise invalid_params("Diarization input is not a valid PCM WAV") from error
    if len(pcm) != frames * 2:
        raise invalid_params("Diarization input has truncated PCM frames")
    return pcm
