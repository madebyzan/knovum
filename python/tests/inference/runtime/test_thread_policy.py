from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest
from test_worker_initialization import initialization

from knovum_inference.runtime.session import _configure_engine_threads
from knovum_inference.runtime.worker import RuntimeInitializeParams
from knovum_inference.transport.validation import validate_params


def test_ctranslate_transcription_initializes_without_torch(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    request = initialization(tmp_path)
    request.params["capability"] = "speech_transcription"
    request.params["deviceProfile"] = {
        "device": "cpu",
        "executionProfileId": "faster-whisper-small-cpu-int8-v1",
        "cpuThreads": 2,
    }
    params = validate_params(RuntimeInitializeParams, request.params)
    monkeypatch.setitem(sys.modules, "torch", None)
    _configure_engine_threads(params)
    assert os.environ["OMP_NUM_THREADS"] == "2"
    assert os.environ["MKL_NUM_THREADS"] == "2"


@pytest.mark.parametrize("device", ["cpu", "cuda", "mps"])
def test_alignment_session_accepts_the_frozen_torch_device(tmp_path: Path, device: str) -> None:
    from knovum_inference.runtime.session import _create_engine
    from knovum_inference.speech.alignment import AlignmentEngine

    request = initialization(tmp_path)
    request.params["capability"] = "speech_alignment"
    request.params["capabilitySpecId"] = "qwen3-forced-aligner-0.6b-hf-v1"
    request.params["modelPaths"] = {"speechAlignment": str(tmp_path)}
    request.params["deviceProfile"] = {
        "device": device,
        "executionProfileId": f"qwen3-forced-aligner-0.6b-hf-{device}-v1",
        "cpuThreads": 2,
    }
    params = validate_params(RuntimeInitializeParams, request.params)
    assert isinstance(_create_engine(params, tmp_path, tmp_path), AlignmentEngine)


@pytest.mark.parametrize("device", ["cpu", "cuda", "mps", "vulkan"])
def test_compiled_alignment_initializes_without_torch(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, device: str
) -> None:
    from knovum_inference.runtime.session import _create_engine
    from knovum_inference.speech.alignment import AlignmentEngine

    request = initialization(tmp_path)
    request.params["capability"] = "speech_alignment"
    request.params["capabilitySpecId"] = "qwen3-forced-aligner-0.6b-q8_0-v1"
    request.params["modelPaths"] = {"speechAlignment": str(tmp_path)}
    request.params["deviceProfile"] = {
        "device": device,
        "executionProfileId": f"qwen3-forced-aligner-0.6b-q8_0-{device}-v1",
        "cpuThreads": 2,
    }
    params = validate_params(RuntimeInitializeParams, request.params)
    monkeypatch.setitem(sys.modules, "torch", None)
    _configure_engine_threads(params)
    assert isinstance(_create_engine(params, tmp_path, tmp_path), AlignmentEngine)
    assert os.environ["OMP_NUM_THREADS"] == "2"
