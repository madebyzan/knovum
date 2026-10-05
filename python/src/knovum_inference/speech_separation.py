from __future__ import annotations

import hashlib
import os
import threading
import wave
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import numpy as np

from knovum_inference.contracts import AudioFileReference, SpeechSeparateParams
from knovum_inference.errors import RpcError, invalid_params, server_error
from knovum_inference.speech_request import attempt_directory, check_deadline, verify_audio

MODEL_SPEC_ID = "spleeter-2stems-fp16-cpu-v1"
SAMPLE_RATE = 44_100
CHANNELS = 2


class SpleeterSourceSeparator:
    def __init__(self, model_directory: Path, work_directory: Path, cpu_threads: int) -> None:
        self._model_directory = model_directory
        self._work_directory = work_directory
        self._cpu_threads = cpu_threads
        self._engine: Any = None

    def separate(self, params: SpeechSeparateParams, cancelled: threading.Event) -> dict[str, Any]:
        if params.model_spec_id != MODEL_SPEC_ID:
            raise invalid_params("Source separation model specification is unsupported")
        source = _read_audio(params.audio_ref)
        check_deadline(params.deadline_ms)
        if cancelled.is_set():
            raise server_error("cancelled", "Source separation was cancelled")
        separation = self._load().process(SAMPLE_RATE, source)
        check_deadline(params.deadline_ms)
        if cancelled.is_set():
            raise server_error("cancelled", "Source separation was cancelled")
        if separation.sample_rate != SAMPLE_RATE or len(separation.stems) != 2:
            raise server_error("invalid_output", "Source separation returned an invalid stem profile")
        vocals = _validated_stem(separation.stems[0].data)
        background = _validated_stem(separation.stems[1].data)
        if vocals.shape != background.shape:
            raise server_error("invalid_output", "Source separation stems have different timelines")
        directory = attempt_directory(self._work_directory, params.request_id)
        vocals_ref = _write_wav(directory, "vocals.wav", vocals, cancelled, params.deadline_ms)
        background_ref = _write_wav(directory, "background.wav", background, cancelled, params.deadline_ms)
        return {
            "requestId": params.request_id,
            "attemptEpoch": params.attempt_epoch,
            "modelSpecId": params.model_spec_id,
            "vocalsRef": vocals_ref,
            "backgroundRef": background_ref,
            "sampleCount": int(background.shape[1]),
        }

    def _load(self) -> Any:
        if self._engine is not None:
            return self._engine
        vocals = self._model_directory / "vocals.fp16.onnx"
        accompaniment = self._model_directory / "accompaniment.fp16.onnx"
        if not vocals.is_file() or not accompaniment.is_file():
            raise server_error("model_corrupt", "Spleeter model snapshot is incomplete")
        try:
            import sherpa_onnx
        except ImportError as error:
            raise server_error("runtime_mismatch", "sherpa-onnx is unavailable") from error
        spleeter = sherpa_onnx.OfflineSourceSeparationSpleeterModelConfig(
            vocals=str(vocals), accompaniment=str(accompaniment)
        )
        model = sherpa_onnx.OfflineSourceSeparationModelConfig(
            spleeter=spleeter, num_threads=self._cpu_threads, debug=False, provider="cpu"
        )
        self._engine = sherpa_onnx.OfflineSourceSeparation(sherpa_onnx.OfflineSourceSeparationConfig(model=model))
        return self._engine


def _read_audio(reference: AudioFileReference) -> np.ndarray[Any, np.dtype[np.float32]]:
    import numpy as np

    path = verify_audio(reference)
    if (
        reference.media_type != "audio/wav"
        or reference.sample_rate != SAMPLE_RATE
        or reference.channels != CHANNELS
        or reference.sample_format != "pcm_s16le"
    ):
        raise invalid_params("Source separation requires 44.1 kHz stereo PCM16 WAV audio")
    try:
        with wave.open(str(path), "rb") as source:
            if (
                source.getframerate() != SAMPLE_RATE
                or source.getnchannels() != CHANNELS
                or source.getsampwidth() != 2
                or source.getcomptype() != "NONE"
            ):
                raise invalid_params("Source separation WAV metadata differs from its reference")
            frame_count = source.getnframes()
            if frame_count == 0:
                raise invalid_params("Source separation audio has no PCM frames")
            if frame_count > SAMPLE_RATE * 10 * 60:
                raise invalid_params("Source separation audio exceeds the ten-minute limit")
            pcm = source.readframes(frame_count)
    except (EOFError, wave.Error) as error:
        raise invalid_params("Source separation audio is not a valid PCM WAV file") from error
    if len(pcm) != frame_count * CHANNELS * 2:
        raise invalid_params("Source separation audio contains an incomplete PCM frame")
    samples = np.frombuffer(pcm, dtype="<i2").reshape(frame_count, CHANNELS)
    return np.ascontiguousarray(samples.T.astype(np.float32) / 32768.0)


def _validated_stem(value: Any) -> np.ndarray[Any, np.dtype[np.float32]]:
    import numpy as np

    samples = np.asarray(value, dtype=np.float32)
    if samples.ndim != 2 or samples.shape[0] != CHANNELS or samples.shape[1] == 0:
        raise server_error("invalid_output", "Source separation returned an invalid stem shape")
    if not np.isfinite(samples).all():
        raise server_error("invalid_output", "Source separation returned non-finite samples")
    return np.ascontiguousarray(samples)


def _write_wav(
    directory: Path,
    file_name: str,
    samples: np.ndarray[Any, np.dtype[np.float32]],
    cancelled: threading.Event,
    deadline_ms: int,
) -> dict[str, Any]:
    import numpy as np

    target = directory / file_name
    temporary = directory / f".{file_name}.part"
    try:
        with wave.open(str(temporary), "wb") as output:
            output.setnchannels(CHANNELS)
            output.setsampwidth(2)
            output.setframerate(SAMPLE_RATE)
            for start in range(0, samples.shape[1], SAMPLE_RATE * 10):
                if cancelled.is_set():
                    raise server_error("cancelled", "Source separation was cancelled")
                check_deadline(deadline_ms)
                pcm_chunk = samples[:, start : start + SAMPLE_RATE * 10].T
                pcm = np.clip(np.rint(pcm_chunk * 32768.0), -32768, 32767).astype("<i2")
                output.writeframesraw(pcm.tobytes())
        with temporary.open("rb+") as output:
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, target)
    except RpcError:
        temporary.unlink(missing_ok=True)
        raise
    except OSError as error:
        temporary.unlink(missing_ok=True)
        raise server_error("resource_exhausted", "Separated audio cannot be written") from error
    digest = hashlib.sha256()
    with target.open("rb") as source:
        while block := source.read(1024 * 1024):
            if cancelled.is_set():
                target.unlink(missing_ok=True)
                raise server_error("cancelled", "Source separation was cancelled")
            check_deadline(deadline_ms)
            digest.update(block)
    return {
        "fileName": f"{directory.name}/{file_name}",
        "mediaType": "audio/wav",
        "byteLength": target.stat().st_size,
        "sha256": digest.hexdigest(),
        "sampleRate": SAMPLE_RATE,
        "channels": CHANNELS,
        "sampleFormat": "pcm_s16le",
    }
