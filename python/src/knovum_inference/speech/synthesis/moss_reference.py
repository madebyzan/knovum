from __future__ import annotations

from typing import Any

from knovum_inference.contracts.generated.audio import AudioFileReference
from knovum_inference.speech.synthesis.reference_audio import read_reference


def reference_codes(engine: Any, reference: AudioFileReference) -> list[list[int]]:
    import numpy as np
    import soxr

    samples = read_reference(reference)
    resampled = soxr.resample(samples, 16_000, 48_000, quality="HQ")
    stereo = np.stack((resampled, resampled), axis=0)[None, :, :]
    codes, lengths = engine.sessions["codec_encode"].run(
        None, {"waveform": stereo, "input_lengths": np.array([stereo.shape[-1]], dtype=np.int32)}
    )
    return codes[0, : int(lengths[0]), :].tolist()  # type: ignore[no-any-return]
