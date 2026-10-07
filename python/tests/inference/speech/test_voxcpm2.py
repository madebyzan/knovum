from __future__ import annotations

import hashlib
import struct
import sys
import threading
import types
import wave
from pathlib import Path
from typing import Any
from uuid import uuid4

import pytest

from knovum_inference.contracts.generated.audio import AudioFileReference
from knovum_inference.contracts.generated.synthesis import (
    SpeechSynthesisEngine,
    SpeechSynthesizeParams,
    VoiceSourceKind,
)
from knovum_inference.errors import RpcError
from knovum_inference.speech.synthesis.engine import SynthesisEngine
from knovum_inference.speech.synthesis.voxcpm2 import _REQUIRED_FILES, VoxCpm2SynthesisEngine


class RecordingPublisher:
    def __init__(self, params: SpeechSynthesizeParams) -> None:
        self._params = params
        self.chunks: list[tuple[list[float], int, str]] = []

    def publish(self, samples: list[float], sample_rate: int, granularity: str) -> None:
        self.chunks.append((list(samples), sample_rate, granularity))

    def finish(self) -> dict[str, object]:
        sample_count = sum(len(samples) for samples, _, _ in self.chunks)
        payload = b"".join(struct.pack("<f", sample) for samples, _, _ in self.chunks for sample in samples)
        return {
            "requestId": self._params.request_id,
            "attemptEpoch": self._params.attempt_epoch,
            "byteLength": len(payload),
            "sha256": hashlib.sha256(payload).hexdigest(),
            "sampleRate": self.chunks[0][1],
            "channels": 1,
            "sampleFormat": "pcm_f32le",
            "sampleCount": sample_count,
        }


def _params(source: dict[str, object], *, style: str | None = None) -> SpeechSynthesizeParams:
    return SpeechSynthesizeParams.model_validate(
        {
            "requestId": str(uuid4()),
            "attemptEpoch": 2,
            "attemptId": str(uuid4()),
            "deadlineMs": 2**63,
            "inputRevision": "speech-revision",
            "modelSpecId": "voxcpm2-v1",
            "ownerId": "speech-segment",
            "ownerKind": "speech_segment",
            "profile": {
                "engine": SpeechSynthesisEngine.vox_cpm2,
                "language": "zh",
                "sampleRate": 48_000,
                "speed": 1.0,
            },
            "seed": 42,
            "style": style,
            "text": "你好",
            "voiceRevision": {"revisionId": str(uuid4()), "speakerId": None} | source,
        }
    )


def _install_fake_vendor(
    monkeypatch: pytest.MonkeyPatch, model_directory: Path, dtype: str
) -> tuple[dict[str, Any], object]:
    calls: dict[str, Any] = {}
    for name in _REQUIRED_FILES:
        (model_directory / name).parent.mkdir(parents=True, exist_ok=True)
        (model_directory / name).touch()

    class FakeWaveform:
        def reshape(self, _: int) -> list[float]:
            return [0.25, -0.25]

    class FakeEngine:
        tts_model = types.SimpleNamespace(config=types.SimpleNamespace(dtype=dtype), sample_rate=48_000)

        def generate_streaming(self, **kwargs: object) -> object:
            calls["generate_streaming"] = kwargs

            def chunks() -> object:
                yield FakeWaveform()

            return chunks()

    engine = FakeEngine()

    class FakeVoxCpm:
        @staticmethod
        def from_pretrained(path: str, **kwargs: object) -> object:
            calls["load"] = (path, kwargs)
            return engine

    voxcpm = types.ModuleType("voxcpm")
    voxcpm.VoxCPM = FakeVoxCpm  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "voxcpm", voxcpm)
    return calls, engine


def test_windows_cpu_profile_uses_frozen_dtype_and_native_generation_controls(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    model_directory = tmp_path / "voxcpm2"
    calls, _ = _install_fake_vendor(monkeypatch, model_directory, "bfloat16")
    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.platform.system", lambda: "Windows")
    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.platform.machine", lambda: "AMD64")

    params = _params(
        {
            "sourceKind": VoiceSourceKind.designed,
            "designPrompt": "calm",
            "referenceAudio": None,
            "referenceText": None,
        }
    )
    publisher = RecordingPublisher(params)
    VoxCpm2SynthesisEngine(model_directory, "cpu").generate(params, publisher, threading.Event())

    assert publisher.chunks == [([0.25, -0.25], 48_000, "model_chunk")]
    assert calls == {
        "load": (
            str(model_directory),
            {"device": "cpu", "load_denoiser": False, "optimize": False, "local_files_only": True},
        ),
        "generate_streaming": {
            "text": "(calm)你好",
            "prompt_wav_path": None,
            "prompt_text": None,
            "reference_wav_path": None,
            "normalize": False,
            "denoise": False,
            "retry_badcase": False,
            "max_len": 4096,
            "seed": 42,
        },
    }


def test_synthesis_engine_routes_generated_vox_enum_to_vox_adapter(tmp_path: Path) -> None:
    params = _params(
        {
            "sourceKind": VoiceSourceKind.designed,
            "designPrompt": "calm",
            "referenceAudio": None,
            "referenceText": None,
        }
    )
    routed: list[SpeechSynthesizeParams] = []

    class FakeVoxEngine:
        def generate(
            self,
            received: SpeechSynthesizeParams,
            publisher: RecordingPublisher,
            cancelled: threading.Event,
        ) -> None:
            routed.append(received)
            assert not cancelled.is_set()
            publisher.publish([0.25, -0.25], 48_000, "model_chunk")

    engine = SynthesisEngine(tmp_path / "model", "cpu", 2)
    engine._voxcpm2 = FakeVoxEngine()  # type: ignore[assignment]
    engine._kokoro._load = lambda: pytest.fail("Vox routing must not load Kokoro")  # type: ignore[method-assign]
    publisher = RecordingPublisher(params)

    result = engine.synthesize(params, publisher, threading.Event())

    assert routed == [params]
    assert result["requestId"] == params.request_id
    assert result["attemptEpoch"] == params.attempt_epoch
    assert result["modelSpecId"] == params.model_spec_id
    assert result["sampleCount"] == 2
    assert result["byteLength"] == 8
    assert result["sampleRate"] == 48_000
    assert result["channels"] == 1
    assert result["sampleFormat"] == "pcm_f32le"
    assert publisher.chunks == [([0.25, -0.25], 48_000, "model_chunk")]


def test_dtype_mismatch_fails_without_generation_fallback(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    model_directory = tmp_path / "voxcpm2"
    calls, _ = _install_fake_vendor(monkeypatch, model_directory, "float32")
    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.platform.system", lambda: "Windows")
    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.platform.machine", lambda: "x86_64")

    params = _params(
        {
            "sourceKind": VoiceSourceKind.designed,
            "designPrompt": "calm",
            "referenceAudio": None,
            "referenceText": None,
        }
    )
    with pytest.raises(RpcError) as caught:
        VoxCpm2SynthesisEngine(model_directory, "cpu").generate(
            params,
            RecordingPublisher(params),
            threading.Event(),
        )
    assert caught.value.stable_code == "runtime_mismatch"
    assert "generate_streaming" not in calls


def test_macos_mps_profile_accepts_only_float32(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    model_directory = tmp_path / "voxcpm2"
    calls, loaded = _install_fake_vendor(monkeypatch, model_directory, "float32")

    assert VoxCpm2SynthesisEngine(model_directory, "mps")._load() is loaded
    assert calls["load"][1]["device"] == "mps"


@pytest.mark.parametrize(("speech_ratio", "accepted"), [(0.59, False), (0.60, True)])
def test_reference_requires_at_least_sixty_percent_speech(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, speech_ratio: float, accepted: bool
) -> None:
    frames = 5 * 16_000
    reference = tmp_path / "reference.wav"
    with wave.open(str(reference), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(16_000)
        output.writeframes(b"\0\0" * frames)
    payload = reference.read_bytes()
    audio_ref = AudioFileReference.model_validate(
        {
            "path": str(reference),
            "mediaType": "audio/wav",
            "byteLength": len(payload),
            "sha256": hashlib.sha256(payload).hexdigest(),
            "sampleRate": 16_000,
            "channels": 1,
            "sampleFormat": "pcm_s16le",
        }
    )

    class FakeArray:
        def astype(self, _dtype: object) -> FakeArray:
            return self

        def __truediv__(self, _value: float) -> FakeArray:
            return self

    fake_numpy = types.ModuleType("numpy")
    fake_numpy.float32 = object()  # type: ignore[attr-defined]
    fake_numpy.frombuffer = lambda *_args, **_kwargs: FakeArray()  # type: ignore[attr-defined]
    fake_silero = types.ModuleType("silero_vad")
    fake_silero.load_silero_vad = lambda **_kwargs: object()  # type: ignore[attr-defined]
    fake_silero.get_speech_timestamps = (  # type: ignore[attr-defined]
        lambda *_args, **_kwargs: [{"start": 0, "end": int(frames * speech_ratio)}]
    )
    monkeypatch.setitem(sys.modules, "numpy", fake_numpy)
    monkeypatch.setitem(sys.modules, "torch", types.SimpleNamespace(from_numpy=lambda samples: samples))
    monkeypatch.setitem(sys.modules, "silero_vad", fake_silero)

    engine = VoxCpm2SynthesisEngine(tmp_path / "model", "cpu")
    if accepted:
        assert engine._verify_reference(audio_ref) == reference
    else:
        with pytest.raises(RpcError) as caught:
            engine._verify_reference(audio_ref)
        assert caught.value.stable_code == "invalid_reference_audio"


def test_exact_transcript_clone_forwards_both_prompt_and_reference_paths(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    model_directory = tmp_path / "voxcpm2"
    calls, _ = _install_fake_vendor(monkeypatch, model_directory, "bfloat16")
    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.platform.system", lambda: "Windows")
    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.platform.machine", lambda: "AMD64")
    reference = tmp_path / "reference.wav"
    engine = VoxCpm2SynthesisEngine(model_directory, "cpu")
    engine._verify_reference = lambda _: reference  # type: ignore[method-assign]
    params = _params(
        {
            "sourceKind": VoiceSourceKind.reference,
            "designPrompt": None,
            "referenceText": "原始台词",
            "referenceAudio": {
                "path": str(reference),
                "mediaType": "audio/wav",
                "byteLength": 1,
                "sha256": "0" * 64,
                "sampleRate": 16_000,
                "channels": 1,
                "sampleFormat": "pcm_s16le",
            },
        }
    )

    publisher = RecordingPublisher(params)
    engine.generate(params, publisher, threading.Event())

    assert calls["generate_streaming"]["text"] == "你好"
    assert calls["generate_streaming"]["prompt_wav_path"] == str(reference)
    assert calls["generate_streaming"]["prompt_text"] == "原始台词"
    assert calls["generate_streaming"]["reference_wav_path"] == str(reference)
