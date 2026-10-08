from __future__ import annotations

import wave

import numpy as np
from numpy.typing import NDArray

from knovum_inference.contracts.generated.audio import AudioFileReference
from knovum_inference.errors import server_error
from knovum_inference.speech.request import verify_audio


def read_reference(reference: AudioFileReference) -> NDArray[np.float32]:
    if (
        reference.media_type != "audio/wav"
        or reference.sample_rate != 16_000
        or reference.channels != 1
        or reference.sample_format != "pcm_s16le"
    ):
        raise server_error("invalid_reference_audio", "Voice reference must be mono PCM16 WAV at 16 kHz")
    path = verify_audio(reference)
    try:
        with wave.open(str(path), "rb") as source:
            frames = source.getnframes()
            if (
                source.getnchannels() != 1
                or source.getsampwidth() != 2
                or source.getframerate() != 16_000
                or source.getcomptype() != "NONE"
                or not 5 * 16_000 <= frames <= 30 * 16_000
            ):
                raise server_error("invalid_reference_audio", "Voice reference WAV metadata is invalid")
            pcm = source.readframes(frames)
    except (OSError, wave.Error) as error:
        raise server_error("invalid_reference_audio", "Voice reference WAV cannot be read") from error
    if len(pcm) != frames * 2:
        raise server_error("invalid_reference_audio", "Voice reference contains truncated PCM frames")
    samples = np.frombuffer(pcm, dtype="<i2").astype(np.float32) / 32768.0
    return samples
