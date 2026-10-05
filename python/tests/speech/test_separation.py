from __future__ import annotations

import hashlib
import threading
import wave
from pathlib import Path
from types import SimpleNamespace
from uuid import uuid4

import numpy as np
import pytest

from knovum_inference.contracts.generated.separation import SpeechSeparateParams
from knovum_inference.errors import RpcError
from knovum_inference.speech.separation import (
    CHANNELS,
    MODEL_SPEC_ID,
    SAMPLE_RATE,
    SpleeterSourceSeparator,
    _read_audio,
    _write_wav,
)


def _audio_reference(path: Path, *, channels: int = CHANNELS) -> dict[str, object]:
    payload = path.read_bytes()
    return {
        "path": str(path),
        "mediaType": "audio/wav",
        "byteLength": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "sampleRate": SAMPLE_RATE,
        "channels": channels,
        "sampleFormat": "pcm_s16le",
    }


def _params(audio_ref: dict[str, object]) -> SpeechSeparateParams:
    return SpeechSeparateParams.model_validate(
        {
            "requestId": str(uuid4()),
            "ownerKind": "speech_source_separation",
            "ownerId": str(uuid4()),
            "attemptId": str(uuid4()),
            "attemptEpoch": 1,
            "inputRevision": "sha256:" + "a" * 64,
            "modelSpecId": MODEL_SPEC_ID,
            "deadlineMs": 2**63,
            "audioRef": audio_ref,
        }
    )


def test_invalid_wav_is_rejected_before_model_load(tmp_path: Path) -> None:
    source = tmp_path / "mono.wav"
    with wave.open(str(source), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(SAMPLE_RATE)
        output.writeframes(b"\0\0" * 32)
    engine = SpleeterSourceSeparator(tmp_path / "model", tmp_path / "work", 2)
    engine._load = lambda: pytest.fail("invalid input must not load sherpa-onnx")  # type: ignore[method-assign]

    with pytest.raises(RpcError) as caught:
        engine.separate(_params(_audio_reference(source)), threading.Event())

    assert caught.value.stable_code == "invalid_output"


def test_actual_frame_limit_is_checked_before_pcm_allocation(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    source = tmp_path / "source.wav"
    source.touch()
    read_called = False

    class OversizedWave:
        def __enter__(self) -> OversizedWave:
            return self

        def __exit__(self, *_args: object) -> None:
            return None

        def getframerate(self) -> int:
            return SAMPLE_RATE

        def getnchannels(self) -> int:
            return CHANNELS

        def getsampwidth(self) -> int:
            return 2

        def getcomptype(self) -> str:
            return "NONE"

        def getnframes(self) -> int:
            return SAMPLE_RATE * 10 * 60 + 1

        def readframes(self, _count: int) -> bytes:
            nonlocal read_called
            read_called = True
            return b""

    monkeypatch.setattr("knovum_inference.speech.separation.verify_audio", lambda _reference: source)
    monkeypatch.setattr("knovum_inference.speech.separation.wave.open", lambda *_args, **_kwargs: OversizedWave())
    reference = SimpleNamespace(
        media_type="audio/wav",
        sample_rate=SAMPLE_RATE,
        channels=CHANNELS,
        sample_format="pcm_s16le",
    )

    with pytest.raises(RpcError) as caught:
        _read_audio(reference)  # type: ignore[arg-type]

    assert caught.value.stable_code == "invalid_output"
    assert read_called is False


def test_pcm_conversion_clips_positive_full_scale_without_wrapping(tmp_path: Path) -> None:
    request_id = str(uuid4())
    directory = tmp_path / f"attempt-{request_id}"
    directory.mkdir()
    samples = np.array(
        [[-1.1, -1.0, 0.0, 0.99999, 1.0, 1.1], [-1.1, -1.0, 0.0, 0.99999, 1.0, 1.1]],
        dtype=np.float32,
    )

    _write_wav(directory, "background.wav", samples, threading.Event(), 2**63)

    with wave.open(str(directory / "background.wav"), "rb") as output:
        pcm = np.frombuffer(output.readframes(output.getnframes()), dtype="<i2")
    assert pcm.min() == -32768
    assert pcm.max() == 32767
    assert pcm[-2:].tolist() == [32767, 32767]
