from __future__ import annotations

import ctypes
import hashlib
import struct
import threading
from pathlib import Path
from typing import Any
from uuid import uuid4

import numpy as np
import pytest

from knovum_inference.contracts.generated.synthesis import (
    SpeechSynthesisEngine,
    SpeechSynthesizeParams,
    VoiceSourceKind,
)
from knovum_inference.errors import RpcError, server_error
from knovum_inference.speech.synthesis.compiled_voice import CompiledVoice
from knovum_inference.speech.synthesis.engine import SynthesisEngine
from knovum_inference.speech.synthesis.voxcpm2 import VoxCpm2SynthesisEngine

ORIGINAL_MODEL_FILES = ("voxcpm2-orig.gguf", "LICENSE", "README.md")


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
            "modelSpecId": "voxcpm2-2b-gguf-orig-v1",
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


def _design_params() -> SpeechSynthesizeParams:
    return _params({"sourceKind": VoiceSourceKind.designed, "designPrompt": "calm"})


def test_compiled_supplier_receives_frozen_model_backend_threads_and_controls(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    calls: list[object] = []
    for name in ORIGINAL_MODEL_FILES:
        (tmp_path / name).touch()

    class FakeCompiled:
        def __init__(self, model: Path, device: str, threads: int) -> None:
            calls.append((model, device, threads))

        def generate(
            self,
            params: SpeechSynthesizeParams,
            text: str,
            reference: object,
            transcript: object,
            publisher: RecordingPublisher,
            cancelled: threading.Event,
        ) -> None:
            calls.append((text, reference, transcript, params.seed))
            publisher.publish([0.25, -0.25], 48_000, "model_chunk")

    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.CompiledVoice", FakeCompiled)
    params = _design_params()
    publisher = RecordingPublisher(params)
    VoxCpm2SynthesisEngine(tmp_path, "mps", params.model_spec_id, 3).generate(params, publisher, threading.Event())
    assert calls == [(tmp_path / "voxcpm2-orig.gguf", "mps", 3), ("(calm)你好", None, None, 42)]
    assert publisher.chunks == [([0.25, -0.25], 48_000, "model_chunk")]


@pytest.mark.parametrize("cancel_after_publish", [False, True])
def test_callback_stops_compiled_work_and_preserves_the_original_python_outcome(cancel_after_publish: bool) -> None:
    cancelled = threading.Event()
    original = server_error("resource_exhausted", "Audio credits could not be published")
    stops: list[int] = []

    class Library:
        def knovum_voice_generate(self, *arguments: Any) -> int:
            samples = np.array([0.25, -0.25], dtype=np.float32)
            status = arguments[-1](samples.ctypes.data_as(ctypes.POINTER(ctypes.c_float)), len(samples), 48_000)
            stops.append(status)
            return status

        def knovum_voice_error(self) -> bytes:
            return b"compiled callback stopped"

    class Publisher:
        def publish(self, *_arguments: Any) -> None:
            if cancel_after_publish:
                cancelled.set()
            else:
                raise original

    voice = CompiledVoice.__new__(CompiledVoice)
    voice.library = Library()
    voice.handle = 1
    with pytest.raises(RpcError) as caught:
        voice.generate(_design_params(), "Hello", None, None, Publisher(), cancelled)
    assert stops == [1]
    if cancel_after_publish:
        assert caught.value.stable_code == "cancelled"
    else:
        assert caught.value is original


def test_unavailable_compiled_backend_does_not_retry_another_backend(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    from knovum_inference.errors import server_error

    calls: list[str] = []
    for name in ORIGINAL_MODEL_FILES:
        (tmp_path / name).touch()

    def unavailable(model: Path, device: str, threads: int) -> None:
        calls.append(device)
        raise server_error("unsupported_device", "GPU is unavailable")

    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.CompiledVoice", unavailable)
    params = _design_params()
    with pytest.raises(RpcError) as caught:
        VoxCpm2SynthesisEngine(tmp_path, "mps", params.model_spec_id).generate(
            params, RecordingPublisher(params), threading.Event()
        )
    assert caught.value.stable_code == "unsupported_device"
    assert calls == ["mps"]


def test_cancelled_request_does_not_load_a_model(tmp_path: Path) -> None:
    cancelled = threading.Event()
    cancelled.set()
    params = _design_params()
    with pytest.raises(RpcError) as caught:
        VoxCpm2SynthesisEngine(tmp_path, "cpu", params.model_spec_id).generate(
            params, RecordingPublisher(params), cancelled
        )
    assert caught.value.stable_code == "cancelled"


@pytest.mark.parametrize("control", ["speed", "sample_rate", "preset"])
def test_compiled_voice_rejects_controls_it_cannot_honor(tmp_path: Path, control: str) -> None:
    params = _design_params()
    if control == "speed":
        params.profile.speed = 1.2
    elif control == "sample_rate":
        params.profile.sample_rate = 24000
    else:
        params.voice_revision.speaker_id = 0
    with pytest.raises(RpcError):
        VoxCpm2SynthesisEngine(tmp_path, "cpu", params.model_spec_id).generate(
            params, RecordingPublisher(params), threading.Event()
        )


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

    engine = SynthesisEngine(tmp_path / "model", "cpu", 2, params.model_spec_id)
    engine._voxcpm2 = FakeVoxEngine()  # type: ignore[assignment]
    assert engine._moss_nano is None
    publisher = RecordingPublisher(params)

    synthesis = engine.synthesize(params, publisher, threading.Event())

    assert routed == [params]
    assert synthesis["requestId"] == params.request_id
    assert synthesis["attemptEpoch"] == params.attempt_epoch
    assert synthesis["modelSpecId"] == params.model_spec_id
    assert synthesis["sampleCount"] == 2
    assert synthesis["byteLength"] == 8
    assert synthesis["sampleRate"] == 48_000
    assert synthesis["channels"] == 1
    assert synthesis["sampleFormat"] == "pcm_f32le"
    assert publisher.chunks == [([0.25, -0.25], 48_000, "model_chunk")]


def test_exact_reference_transcript_excludes_style_guidance(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    reference = np.array([0.2, -0.2], dtype=np.float32)
    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.read_reference", lambda _: reference)
    params = _params(
        {
            "sourceKind": VoiceSourceKind.reference,
            "referenceAudio": _reference_audio(tmp_path),
            "referenceText": "Reference words",
        }
    )
    engine = VoxCpm2SynthesisEngine(tmp_path, "cpu", params.model_spec_id)
    text, received, transcript = engine._generation_inputs(params)
    assert text == "你好"
    assert received is not None
    np.testing.assert_array_equal(received, reference)
    assert transcript == "Reference words"
    params.style = "warm"
    with pytest.raises(RpcError):
        engine._generation_inputs(params)


def test_reference_style_and_saved_design_exact_clone_use_the_frozen_reference_audio(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    reference = np.array([0.2, -0.2], dtype=np.float32)
    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.read_reference", lambda _: reference)
    engine = VoxCpm2SynthesisEngine(tmp_path, "cpu", "voxcpm2-2b-gguf-orig-v1")
    styled = _params(
        {
            "sourceKind": VoiceSourceKind.reference,
            "referenceAudio": _reference_audio(tmp_path),
            "referenceText": None,
        },
        style="warm",
    )
    saved_design = _params(
        {
            "sourceKind": VoiceSourceKind.reference,
            "referenceAudio": _reference_audio(tmp_path),
            "referenceText": "Candidate sample",
            "designPrompt": None,
        }
    )
    styled_text, styled_reference, styled_transcript = engine._generation_inputs(styled)
    assert styled_text == "(warm)你好"
    assert styled_reference is not None
    np.testing.assert_array_equal(styled_reference, reference)
    assert styled_transcript is None
    saved_text, saved_reference, saved_transcript = engine._generation_inputs(saved_design)
    assert saved_text == "你好"
    assert saved_reference is not None
    np.testing.assert_array_equal(saved_reference, reference)
    assert saved_transcript == "Candidate sample"


def _reference_audio(tmp_path: Path) -> dict[str, object]:
    return {
        "path": str(tmp_path / "reference.wav"),
        "mediaType": "audio/wav",
        "byteLength": 32044,
        "sha256": "a" * 64,
        "sampleRate": 16000,
        "channels": 1,
        "sampleFormat": "pcm_s16le",
    }


def test_q8_spec_loads_only_the_q8_weight(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    params = _design_params()
    params.model_spec_id = "voxcpm2-2b-gguf-q8_0-v1"
    for name in ("voxcpm2-q8_0.gguf", "LICENSE", "README.md"):
        (tmp_path / name).touch()
    loaded: list[Path] = []

    class FakeCompiled:
        def __init__(self, model: Path, _device: str, _threads: int) -> None:
            loaded.append(model)

        def generate(
            self,
            _params: SpeechSynthesizeParams,
            _text: str,
            _reference: object,
            _transcript: object,
            publisher: RecordingPublisher,
            _cancelled: threading.Event,
        ) -> None:
            publisher.publish([0.1], 48_000, "model_chunk")

    monkeypatch.setattr("knovum_inference.speech.synthesis.voxcpm2.CompiledVoice", FakeCompiled)
    VoxCpm2SynthesisEngine(tmp_path, "cpu", params.model_spec_id).generate(
        params, RecordingPublisher(params), threading.Event()
    )
    assert loaded == [tmp_path / "voxcpm2-q8_0.gguf"]
