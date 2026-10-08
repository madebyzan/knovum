from __future__ import annotations

import threading
import wave
from pathlib import Path
from typing import Any

from knovum_inference.contracts.generated.diarization import SpeechDiarizeParams
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech.request import check_deadline, milliseconds, verify_audio


class DiarizationEngine:
    def __init__(self, model_directory: Path, device: str, cpu_threads: int, model_spec_id: str) -> None:
        if model_spec_id not in {"sherpa-diarization-cpu-v1", "sherpa-diarization-fp32-cpu-v1"}:
            raise server_error("runtime_mismatch", "Speaker diarization model specification is unsupported")
        self._model_spec_id = model_spec_id
        self._model_directory = model_directory
        self._device = device
        self._cpu_threads = cpu_threads
        self._pipeline: Any = None
        self._config: Any = None

    def diarize(self, params: SpeechDiarizeParams, cancelled: threading.Event) -> dict[str, Any]:
        def observe(_processed: int, _total: int) -> int:
            if cancelled.is_set():
                raise server_error("cancelled", "Speaker diarization was cancelled")
            check_deadline(params.deadline_ms)
            return 0

        observe(0, 0)
        if params.model_spec_id != self._model_spec_id:
            raise server_error("runtime_mismatch", "Speaker diarization request does not match the frozen model")
        if self._device != "cpu":
            raise server_error("unsupported_device", "The Sherpa diarization profile requires CPU")
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
        if bounds.minimum != bounds.maximum or (bounds.minimum is not None and not 1 <= bounds.minimum <= 32):
            raise invalid_params("Specify the known speaker count from one to 32, or use automatic detection")
        pcm = _read_pcm(audio_path)
        if self._pipeline is None:
            self._load()
        import numpy as np
        import sherpa_onnx

        self._config.clustering = sherpa_onnx.FastClusteringConfig(num_clusters=bounds.minimum or -1, threshold=0.5)
        self._pipeline.set_config(self._config)
        waveform = np.frombuffer(pcm, dtype="<i2").astype(np.float32) / 32768
        del pcm
        observe(0, 0)

        output = self._pipeline.process(waveform, callback=observe).sort_by_start_time()
        turns = [
            {"speakerId": str(turn.speaker), "startMs": milliseconds(turn.start), "endMs": milliseconds(turn.end)}
            for turn in output
        ]
        observe(0, 0)
        return {
            "requestId": params.request_id,
            "attemptEpoch": params.attempt_epoch,
            "modelSpecId": params.model_spec_id,
            "turns": turns,
        }

    def _load(self) -> None:
        if not all((self._model_directory / name).is_file() for name in ("segmentation.onnx", "embedding.onnx")):
            raise server_error("model_corrupt", "Speaker diarization model directory is incomplete")
        try:
            import sherpa_onnx
        except ImportError as error:
            raise server_error("runtime_mismatch", "Sherpa-ONNX is unavailable") from error
        config = sherpa_onnx.OfflineSpeakerDiarizationConfig(
            segmentation=sherpa_onnx.OfflineSpeakerSegmentationModelConfig(
                pyannote=sherpa_onnx.OfflineSpeakerSegmentationPyannoteModelConfig(
                    model=str(self._model_directory / "segmentation.onnx"),
                ),
                num_threads=self._cpu_threads,
                provider="cpu",
            ),
            embedding=sherpa_onnx.SpeakerEmbeddingExtractorConfig(
                model=str(self._model_directory / "embedding.onnx"),
                num_threads=self._cpu_threads,
                provider="cpu",
            ),
            clustering=sherpa_onnx.FastClusteringConfig(num_clusters=-1, threshold=0.5),
            min_duration_on=0.3,
            min_duration_off=0.5,
        )
        if not config.validate():
            raise server_error("model_corrupt", "Speaker diarization model configuration is invalid")
        self._config = config
        self._pipeline = sherpa_onnx.OfflineSpeakerDiarization(config)


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
