from __future__ import annotations

import hashlib
import sys
import threading
import types
import wave
from contextlib import nullcontext
from pathlib import Path
from typing import Any
from uuid import uuid4

import pytest

from knovum_inference.contracts.generated.alignment import SpeechAlignParams
from knovum_inference.contracts.generated.diarization import SpeechDiarizeParams
from knovum_inference.errors import RpcError
from knovum_inference.speech.alignment import AlignmentEngine
from knovum_inference.speech.diarization import DiarizationEngine


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
        "modelSpecId": "sherpa-diarization-cpu-v1",
        "ownerId": "media-node",
        "ownerKind": "media_transcript",
    }


@pytest.mark.parametrize("device", ["cpu", "cuda", "mps"])
def test_alignment_rejects_unsupported_language_without_loading_model(tmp_path: Path, device: str) -> None:
    params = SpeechAlignParams.model_validate(
        _identity(_audio(tmp_path / "source.wav"))
        | {
            "modelSpecId": "qwen3-forced-aligner-0.6b-hf-v1",
            "profile": {"device": device, "language": "xx"},
            "textSegments": [{"segmentId": "segment-1", "startMs": 0, "endMs": 500, "text": "hello"}],
        }
    )
    engine = AlignmentEngine(tmp_path / "model", device, 2, "qwen3-forced-aligner-0.6b-hf-v1")
    engine._load = lambda: pytest.fail("unsupported languages must not load a model")  # type: ignore[method-assign]

    with pytest.raises(RpcError) as caught:
        engine.align(params, threading.Event())
    assert caught.value.stable_code == "unsupported_language"


@pytest.mark.parametrize("device", ["cpu", "cuda", "mps"])
@pytest.mark.parametrize("loading_issue", [None, "missing_keys", "unexpected_keys", "mismatched_keys", "error_msgs"])
def test_alignment_loader_is_local_float32_on_the_frozen_device(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, loading_issue: str | None, device: str
) -> None:
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
        def from_pretrained(path: Path, **kwargs: object) -> tuple[FakeModel, dict[str, list[str]]]:
            calls["model"] = (path, kwargs)
            loading = {key: [] for key in ("missing_keys", "unexpected_keys", "mismatched_keys", "error_msgs")}
            if loading_issue is not None:
                loading[loading_issue] = ["model-weight-does-not-match"]
            return FakeModel(), loading

    fake_torch = types.SimpleNamespace(
        float32="float32",
        inference_mode=lambda: nullcontext(),
    )
    fake_transformers = types.SimpleNamespace(
        AutoModelForTokenClassification=FakeModelFactory,
        AutoProcessor=FakeProcessorFactory,
    )
    monkeypatch.setitem(sys.modules, "torch", fake_torch)
    monkeypatch.setitem(sys.modules, "transformers", fake_transformers)

    model_directory = tmp_path / "alignment"
    if loading_issue is not None:
        with pytest.raises(RpcError) as caught:
            AlignmentEngine(model_directory, device, 6, "qwen3-forced-aligner-0.6b-hf-v1")._load()
        assert caught.value.stable_code == "model_corrupt"
        assert "device" not in calls
        return
    AlignmentEngine(model_directory, device, 6, "qwen3-forced-aligner-0.6b-hf-v1")._load()

    assert calls == {
        "processor": (model_directory, {"local_files_only": True}),
        "model": (model_directory, {"local_files_only": True, "dtype": "float32", "output_loading_info": True}),
        "device": device,
        "eval": True,
    }


def test_diarization_rejects_incomplete_offline_snapshot_before_vendor_import(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    params = SpeechDiarizeParams.model_validate(
        _identity(_audio(tmp_path / "source.wav")) | {"speakerCountBounds": {"minimum": 2, "maximum": 2}}
    )
    monkeypatch.delitem(sys.modules, "pyannote.audio", raising=False)

    with pytest.raises(RpcError) as caught:
        DiarizationEngine(tmp_path / "missing-model", "cpu", 4, "sherpa-diarization-cpu-v1").diarize(
            params, threading.Event()
        )
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
    engine = DiarizationEngine(tmp_path / "model", "cpu", 4, "sherpa-diarization-cpu-v1")
    engine._pipeline = lambda *_args, **_kwargs: pytest.fail("invalid bounds must not invoke the pipeline")

    with pytest.raises(RpcError) as caught:
        engine.diarize(params, threading.Event())
    assert caught.value.stable_code == "invalid_output"


@pytest.mark.parametrize("model_spec", ["sherpa-diarization-cpu-v1", "sherpa-diarization-fp32-cpu-v1"])
def test_diarization_uses_public_cpu_models_and_known_speaker_count(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, model_spec: str
) -> None:
    calls: dict[str, Any] = {}
    model_directory = tmp_path / "diarization"
    model_directory.mkdir()
    for name in ("segmentation.onnx", "embedding.onnx"):
        (model_directory / name).touch()

    class Pipeline:
        def __init__(self, config: object) -> None:
            calls["initial"] = config

        def set_config(self, config: object) -> None:
            calls["clustering"] = config

        def process(self, waveform: Any, *, callback: Any) -> Any:
            assert waveform.dtype.name == "float32"
            assert waveform.shape == (16000,)
            assert callback(1, 1) == 0
            return types.SimpleNamespace(
                sort_by_start_time=lambda: [types.SimpleNamespace(start=0.1, end=0.8, speaker=1)]
            )

    def config(**kwargs: object) -> Any:
        return types.SimpleNamespace(**kwargs, validate=lambda: True)

    monkeypatch.setitem(
        sys.modules,
        "sherpa_onnx",
        types.SimpleNamespace(
            OfflineSpeakerDiarizationConfig=config,
            OfflineSpeakerSegmentationModelConfig=config,
            OfflineSpeakerSegmentationPyannoteModelConfig=config,
            SpeakerEmbeddingExtractorConfig=config,
            FastClusteringConfig=config,
            OfflineSpeakerDiarization=Pipeline,
        ),
    )
    params = SpeechDiarizeParams.model_validate(
        _identity(_audio(tmp_path / "source.wav"))
        | {"modelSpecId": model_spec, "speakerCountBounds": {"minimum": 3, "maximum": 3}}
    )
    engine = DiarizationEngine(model_directory, "cpu", 2, model_spec)
    result = engine.diarize(params, threading.Event())
    assert calls["initial"].segmentation.pyannote.model == str(model_directory / "segmentation.onnx")
    assert calls["initial"].embedding.model == str(model_directory / "embedding.onnx")
    assert calls["initial"].embedding.provider == "cpu"
    assert calls["initial"].embedding.num_threads == 2
    assert calls["clustering"].clustering.num_clusters == 3
    assert result["turns"] == [{"speakerId": "1", "startMs": 100, "endMs": 800}]


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
            "modelSpecId": "qwen3-forced-aligner-0.6b-hf-v1",
            "profile": {"device": "cpu", "language": "en"},
            "textSegments": [{"segmentId": "s", "startMs": 0, "endMs": 1001, "text": "hello"}],
        }
    )
    engine = AlignmentEngine(tmp_path / "model", "cpu", 2, "qwen3-forced-aligner-0.6b-hf-v1")
    engine._load = lambda: pytest.fail("invalid segment must not load model")  # type: ignore[method-assign]
    with pytest.raises(RpcError):
        engine.align(params, threading.Event())


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
    params = SpeechDiarizeParams.model_validate(_identity(audio) | {"speakerCountBounds": {"minimum": 2, "maximum": 2}})
    engine = DiarizationEngine(tmp_path / "model", "cpu", 4, "sherpa-diarization-cpu-v1")
    engine._pipeline = lambda *_args, **_kwargs: pytest.fail("truncated PCM must not reach model")
    with pytest.raises(RpcError, match="truncated"):
        engine.diarize(params, threading.Event())


def test_diarization_cancelled_before_loading_never_opens_the_model(tmp_path: Path) -> None:
    params = SpeechDiarizeParams.model_validate(_identity(_audio(tmp_path / "source.wav")) | {"speakerCountBounds": {}})
    cancelled = threading.Event()
    cancelled.set()
    engine = DiarizationEngine(tmp_path / "model", "cpu", 4, "sherpa-diarization-cpu-v1")
    engine._load = lambda: pytest.fail("cancelled request must not load the model")  # type: ignore[method-assign]
    with pytest.raises(RpcError) as caught:
        engine.diarize(params, cancelled)
    assert caught.value.stable_code == "cancelled"


def test_diarization_processing_callback_observes_cancellation(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    cancelled = threading.Event()

    class Pipeline:
        def set_config(self, config: object) -> None:
            pass

        def process(self, waveform: Any, *, callback: Any) -> Any:
            cancelled.set()
            callback(1, 10)
            pytest.fail("native processing must stop after cancellation")

    monkeypatch.setitem(sys.modules, "sherpa_onnx", types.SimpleNamespace(FastClusteringConfig=lambda **kwargs: kwargs))
    params = SpeechDiarizeParams.model_validate(_identity(_audio(tmp_path / "source.wav")) | {"speakerCountBounds": {}})
    engine = DiarizationEngine(tmp_path / "model", "cpu", 4, "sherpa-diarization-cpu-v1")
    engine._pipeline = Pipeline()
    engine._config = types.SimpleNamespace(clustering=None)
    with pytest.raises(RpcError) as caught:
        engine.diarize(params, cancelled)
    assert caught.value.stable_code == "cancelled"


@pytest.mark.parametrize("model_spec", ["sherpa-diarization-cpu-v1", "sherpa-diarization-fp32-cpu-v1"])
def test_diarization_rejects_a_request_for_another_frozen_model(tmp_path: Path, model_spec: str) -> None:
    params = SpeechDiarizeParams.model_validate(
        _identity(_audio(tmp_path / "source.wav")) | {"modelSpecId": "unknown-model", "speakerCountBounds": {}}
    )
    engine = DiarizationEngine(tmp_path / "model", "cpu", 2, model_spec)
    engine._load = lambda: pytest.fail("model mismatch must not load a snapshot")  # type: ignore[method-assign]
    with pytest.raises(RpcError) as caught:
        engine.diarize(params, threading.Event())
    assert caught.value.stable_code == "runtime_mismatch"
