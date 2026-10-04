from __future__ import annotations

import hashlib
import sys
import types
import wave
from contextlib import nullcontext
from pathlib import Path
from typing import Any
from unittest.mock import Mock
from uuid import uuid4

import pytest

from knovum_inference.contracts import SpeechAlignParams, SpeechDiarizeParams
from knovum_inference.errors import RpcError
from knovum_inference.speech_alignment import AlignmentEngine
from knovum_inference.speech_diarization import DiarizationEngine


def _audio(path: Path, duration_ms: int = 1_000) -> dict[str, object]:
    with wave.open(str(path), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(16_000)
        output.writeframes(b"\0\0" * (16_000 * duration_ms // 1_000))
    payload = path.read_bytes()
    return {
        "path": str(path),
        "mediaType": "audio/wav",
        "byteLength": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "sampleRate": 16_000,
        "channels": 1,
        "sampleFormat": "pcm_s16le",
    }


def _identity(audio_ref: dict[str, object]) -> dict[str, object]:
    return {
        "requestId": str(uuid4()),
        "attemptEpoch": 3,
        "attemptId": str(uuid4()),
        "audioRef": audio_ref,
        "deadlineMs": 2**63,
        "inputRevision": "source-revision",
        "modelSpecId": "model-v1",
        "ownerId": "media-node",
        "ownerKind": "media_transcript",
    }


def test_alignment_rejects_unsupported_language_without_loading_model(tmp_path: Path) -> None:
    params = SpeechAlignParams.model_validate(
        _identity(_audio(tmp_path / "source.wav"))
        | {
            "profile": {"device": "cpu", "language": "xx"},
            "textSegments": [{"segmentId": "segment-1", "startMs": 0, "endMs": 500, "text": "hello"}],
        }
    )
    engine = AlignmentEngine(tmp_path / "model", "cpu", 2)
    engine._load = lambda: pytest.fail("unsupported languages must not load a model")  # type: ignore[method-assign]

    with pytest.raises(RpcError) as caught:
        engine.align(params)
    assert caught.value.stable_code == "unsupported_language"


def test_alignment_loader_is_local_cpu_float32(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    calls: dict[str, Any] = {}

    class FakeModel:
        def to(self, device: str) -> FakeModel:
            calls["device"] = device
            return self

        def eval(self) -> FakeModel:
            calls["eval"] = True
            return self

    class FakeProcessorFactory:
        @staticmethod
        def from_pretrained(path: Path, **kwargs: object) -> object:
            calls["processor"] = (path, kwargs)
            return object()

    class FakeModelFactory:
        @staticmethod
        def from_pretrained(path: Path, **kwargs: object) -> FakeModel:
            calls["model"] = (path, kwargs)
            return FakeModel()

    fake_torch = types.SimpleNamespace(
        float32="float32",
        inference_mode=lambda: nullcontext(),
        set_num_threads=lambda threads: calls.setdefault("threads", threads),
    )
    fake_transformers = types.SimpleNamespace(
        AutoModelForTokenClassification=FakeModelFactory,
        AutoProcessor=FakeProcessorFactory,
    )
    monkeypatch.setitem(sys.modules, "torch", fake_torch)
    monkeypatch.setitem(sys.modules, "transformers", fake_transformers)

    model_directory = tmp_path / "alignment"
    AlignmentEngine(model_directory, "cpu", 6)._load()

    assert calls == {
        "threads": 6,
        "processor": (model_directory, {"local_files_only": True}),
        "model": (model_directory, {"local_files_only": True, "dtype": "float32"}),
        "device": "cpu",
        "eval": True,
    }


def test_diarization_rejects_incomplete_offline_snapshot_before_vendor_import(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    params = SpeechDiarizeParams.model_validate(
        _identity(_audio(tmp_path / "source.wav")) | {"speakerCountBounds": {"minimum": 1, "maximum": 2}}
    )
    monkeypatch.delitem(sys.modules, "pyannote.audio", raising=False)

    with pytest.raises(RpcError) as caught:
        DiarizationEngine(tmp_path / "missing-model", "cpu").diarize(params)
    assert caught.value.stable_code == "model_corrupt"


@pytest.mark.parametrize(
    "bounds",
    [
        {"minimum": 0, "maximum": 2},
        {"minimum": 1, "maximum": 33},
        {"minimum": 4, "maximum": 2},
    ],
)
def test_diarization_rejects_invalid_speaker_bounds_before_loading_model(
    tmp_path: Path, bounds: dict[str, int]
) -> None:
    params = SpeechDiarizeParams.model_validate(
        _identity(_audio(tmp_path / "source.wav")) | {"speakerCountBounds": bounds}
    )
    engine = DiarizationEngine(tmp_path / "model", "cpu")
    engine._pipeline = lambda *_args, **_kwargs: pytest.fail("invalid bounds must not invoke the pipeline")

    with pytest.raises(RpcError) as caught:
        engine.diarize(params)
    assert caught.value.stable_code == "invalid_output"


def test_diarization_loads_owned_config_and_forwards_speaker_bounds(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    calls: dict[str, Any] = {}
    model_directory = tmp_path / "diarization"
    for relative in (
        "config.yaml",
        "embedding/pytorch_model.bin",
        "segmentation/pytorch_model.bin",
        "plda/plda.npz",
        "plda/xvec_transform.npz",
    ):
        path = model_directory / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.touch()

    class FakeDiarization:
        def itertracks(self, *, yield_label: bool) -> list[tuple[object, None, str]]:
            assert yield_label is True
            return [(types.SimpleNamespace(start=0.1, end=0.8), None, "speaker-1")]

    class FakePipeline:
        @classmethod
        def from_pretrained(cls, path: str) -> FakePipeline:
            calls["config"] = path
            return cls()

        def to(self, device: object) -> None:
            calls["device"] = device

        def __call__(self, audio: dict[str, object], **kwargs: int) -> object:
            calls["audio"] = audio
            calls["bounds"] = kwargs
            return types.SimpleNamespace(speaker_diarization=FakeDiarization())

    pyannote_audio = types.ModuleType("pyannote.audio")
    pyannote_audio.Pipeline = FakePipeline  # type: ignore[attr-defined]
    normalized = object()
    tensor = Mock()
    tensor.to.return_value.div_.return_value.unsqueeze.return_value = normalized
    frombuffer = Mock(return_value=tensor)
    monkeypatch.setitem(
        sys.modules,
        "torch",
        types.SimpleNamespace(
            device=lambda value: f"device:{value}", frombuffer=frombuffer, int16="int16", float32="float32"
        ),
    )
    monkeypatch.setitem(sys.modules, "pyannote", types.ModuleType("pyannote"))
    monkeypatch.setitem(sys.modules, "pyannote.audio", pyannote_audio)
    audio_ref = _audio(tmp_path / "source.wav")
    params = SpeechDiarizeParams.model_validate(
        _identity(audio_ref) | {"speakerCountBounds": {"minimum": 1, "maximum": 3}}
    )

    result = DiarizationEngine(model_directory, "cpu").diarize(params)

    assert calls == {
        "config": str(model_directory / "config.yaml"),
        "device": "device:cpu",
        "audio": {"waveform": normalized, "sample_rate": 16000, "uri": "source"},
        "bounds": {"min_speakers": 1, "max_speakers": 3},
    }
    assert frombuffer.call_args.kwargs == {"dtype": "int16"}
    assert len(frombuffer.call_args.args[0]) == 32000
    assert result == {
        "requestId": params.request_id,
        "attemptEpoch": 3,
        "modelSpecId": "model-v1",
        "turns": [{"speakerId": "speaker-1", "startMs": 100, "endMs": 800}],
    }


def test_alignment_clip_uses_selected_normalized_pcm_frames(tmp_path: Path) -> None:
    import numpy as np

    path = tmp_path / "clip.wav"
    samples = np.array([0] * 16 + [-32768, 32767] + [0] * 14, dtype="<i2")
    with wave.open(str(path), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(16000)
        output.writeframes(samples.tobytes())
    with wave.open(str(path), "rb") as source:
        clip = AlignmentEngine._read_clip(source, 1, 2)
    assert clip.dtype == np.float32
    assert clip.shape == (16,)
    assert clip[0] == -1.0
    assert clip[1] == 32767 / 32768


def test_alignment_outside_audio_rejected_before_model_load(tmp_path: Path) -> None:
    params = SpeechAlignParams.model_validate(
        _identity(_audio(tmp_path / "source.wav"))
        | {
            "profile": {"device": "cpu", "language": "en"},
            "textSegments": [{"segmentId": "s", "startMs": 0, "endMs": 1001, "text": "hello"}],
        }
    )
    engine = AlignmentEngine(tmp_path / "model", "cpu", 2)
    engine._load = lambda: pytest.fail("invalid segment must not load model")  # type: ignore[method-assign]
    with pytest.raises(RpcError):
        engine.align(params)


def test_diarization_truncated_pcm_is_rejected_before_vendor_import(tmp_path: Path) -> None:
    path = tmp_path / "source.wav"
    _audio(path)
    path.write_bytes(path.read_bytes()[:-2])
    payload = path.read_bytes()
    audio = {
        "path": str(path),
        "mediaType": "audio/wav",
        "byteLength": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "sampleRate": 16000,
        "channels": 1,
        "sampleFormat": "pcm_s16le",
    }
    params = SpeechDiarizeParams.model_validate(_identity(audio) | {"speakerCountBounds": {"minimum": 1, "maximum": 2}})
    engine = DiarizationEngine(tmp_path / "model", "cpu")
    engine._pipeline = lambda *_args, **_kwargs: pytest.fail("truncated PCM must not reach model")
    with pytest.raises(RpcError, match="truncated"):
        engine.diarize(params)
