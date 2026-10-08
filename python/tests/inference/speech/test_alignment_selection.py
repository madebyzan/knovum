from __future__ import annotations

import sys
import threading
import types
from pathlib import Path
from typing import Any

import pytest
from test_enrichment import _audio, _identity

from knovum_inference.contracts.generated.alignment import SpeechAlignParams
from knovum_inference.errors import RpcError, server_error
from knovum_inference.speech.alignment import AlignmentEngine

NATIVE_SPEC = "qwen3-forced-aligner-0.6b-q8_0-v1"
FULL_SPEC = "qwen3-forced-aligner-0.6b-hf-v1"


def request(path: Path, device: str = "cpu", spec: str = NATIVE_SPEC) -> SpeechAlignParams:
    return SpeechAlignParams.model_validate(
        _identity(_audio(path))
        | {
            "modelSpecId": spec,
            "profile": {"language": "en", "device": device},
            "textSegments": [{"segmentId": "segment", "startMs": 0, "endMs": 1000, "text": "Hello"}],
        }
    )


def test_changing_default_cannot_change_the_existing_alignment_model(tmp_path: Path) -> None:
    engine = AlignmentEngine(tmp_path, "cpu", 2, NATIVE_SPEC)
    with pytest.raises(RpcError, match="frozen model"):
        engine.align(request(tmp_path / "source.wav", spec=FULL_SPEC), threading.Event())


def test_cancelled_alignment_never_loads_a_model(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setitem(
        sys.modules,
        "knovum_inference.speech.native_alignment",
        types.SimpleNamespace(NativeAlignment=lambda *_args: pytest.fail("cancelled request loaded model")),
    )
    cancelled = threading.Event()
    cancelled.set()
    engine = AlignmentEngine(tmp_path, "cpu", 2, NATIVE_SPEC)
    with pytest.raises(RpcError) as caught:
        engine.align(request(tmp_path / "source.wav"), cancelled)
    assert caught.value.stable_code == "cancelled"


def test_gpu_model_failure_is_reported_without_trying_another_engine(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    devices: list[str] = []

    def fail_gpu(_path: Path, device: str, _threads: int) -> Any:
        devices.append(device)
        raise server_error("model_load_failed", "Frozen GPU model cannot load")

    monkeypatch.setitem(
        sys.modules,
        "knovum_inference.speech.native_alignment",
        types.SimpleNamespace(NativeAlignment=fail_gpu),
    )
    engine = AlignmentEngine(tmp_path, "mps", 2, NATIVE_SPEC)
    monkeypatch.setattr(engine, "_load", lambda: pytest.fail("native failure invoked Transformers"))
    with pytest.raises(RpcError) as caught:
        engine.align(request(tmp_path / "source.wav", device="mps"), threading.Event())
    assert caught.value.stable_code == "model_load_failed"
    assert devices == ["mps"]


def test_native_alignment_preserves_exact_selected_pcm_and_absolute_timing(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    calls: dict[str, Any] = {}

    class Native:
        def __init__(self, path: Path, device: str, threads: int) -> None:
            calls["model"] = (path, device, threads)

        def align(self, samples: Any, text: str, language: str) -> list[dict[str, Any]]:
            calls["input"] = (len(samples), text, language)
            return [{"text": "Hello", "start_time": 0.1, "end_time": 0.5}]

    monkeypatch.setitem(
        sys.modules,
        "knovum_inference.speech.native_alignment",
        types.SimpleNamespace(NativeAlignment=Native),
    )
    params = request(tmp_path / "source.wav")
    params.text_segments[0].start_ms = 200
    params.text_segments[0].end_ms = 800
    engine = AlignmentEngine(tmp_path, "cpu", 2, NATIVE_SPEC)
    result = engine.align(params, threading.Event())
    assert calls["model"] == (tmp_path / "qwen3-forced-aligner-0.6b-q8_0.gguf", "cpu", 2)
    assert calls["input"] == (9600, "Hello", "English")
    assert result["modelSpecId"] == NATIVE_SPEC
    assert result["coverage"] == 1.0
    assert result["units"] == [
        {
            "segmentId": "segment",
            "text": "Hello",
            "unitKind": "word",
            "startMs": 300,
            "endMs": 700,
            "score": None,
            "unalignedReason": None,
        }
    ]
