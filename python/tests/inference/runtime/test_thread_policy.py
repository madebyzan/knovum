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
