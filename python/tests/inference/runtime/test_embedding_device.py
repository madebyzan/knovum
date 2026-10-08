from __future__ import annotations

import os
import sys
import types
from pathlib import Path
from typing import Any

import pytest

from knovum_inference.embedding import REQUIRED_MODEL_FILES, EmbeddingEngine
from knovum_inference.errors import RpcError


@pytest.mark.parametrize("device", ["cpu", "cuda", "mps"])
@pytest.mark.parametrize(
    "pooling_mode,include_prompt",
    [("lasttoken", True), ("mean", True), (("lasttoken", "mean"), True), ("lasttoken", False)],
)
def test_frozen_embedding_loads_on_the_selected_device_without_reconfiguring_threads(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    device: str,
    pooling_mode: str | tuple[str, ...],
    include_prompt: bool,
) -> None:
    monkeypatch.setenv("OMP_NUM_THREADS", "session-owned-omp")
    monkeypatch.setenv("MKL_NUM_THREADS", "session-owned-mkl")
    for relative in REQUIRED_MODEL_FILES:
        path = tmp_path / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("fixture")
    calls: list[dict[str, Any]] = []

    class Pooling:
        def __init__(self) -> None:
            self.pooling_mode = pooling_mode
            self.include_prompt = include_prompt

    class Model:
        max_seq_length = 0

        def __init__(self, directory: str, **kwargs: Any) -> None:
            assert directory == str(tmp_path)
            calls.append(kwargs)

        def get_embedding_dimension(self) -> int:
            return 1024

        def modules(self) -> list[Pooling]:
            return [Pooling()]

    monkeypatch.setitem(sys.modules, "torch", types.SimpleNamespace(float32="f32"))
    monkeypatch.setitem(
        sys.modules,
        "sentence_transformers",
        types.SimpleNamespace(SentenceTransformer=Model),
    )
    monkeypatch.setitem(
        sys.modules, "sentence_transformers.sentence_transformer.modules", types.SimpleNamespace(Pooling=Pooling)
    )
    engine = EmbeddingEngine(str(tmp_path), device)
    if pooling_mode != "lasttoken" or not include_prompt:
        with pytest.raises(RpcError) as caught:
            engine._load()
        assert caught.value.stable_code == "model_corrupt"
        return
    loaded = engine._load()
    assert engine._load() is loaded
    assert calls == [
        {
            "device": device,
            "local_files_only": True,
            "trust_remote_code": False,
            "model_kwargs": {"dtype": "f32"},
        }
    ]
    assert loaded.max_seq_length == 2048
    assert os.environ["OMP_NUM_THREADS"] == "session-owned-omp"
    assert os.environ["MKL_NUM_THREADS"] == "session-owned-mkl"


def test_failed_gpu_load_is_not_retried_on_cpu(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    for relative in REQUIRED_MODEL_FILES:
        path = tmp_path / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.touch()
    attempts: list[str] = []

    def fail(directory: str, **kwargs: Any) -> None:
        attempts.append(kwargs["device"])
        raise RuntimeError("GPU model load failed")

    monkeypatch.setitem(sys.modules, "torch", types.SimpleNamespace(float32="f32"))
    monkeypatch.setitem(sys.modules, "sentence_transformers", types.SimpleNamespace(SentenceTransformer=fail))
    monkeypatch.setitem(
        sys.modules, "sentence_transformers.sentence_transformer.modules", types.SimpleNamespace(Pooling=object)
    )
    with pytest.raises(RpcError):
        EmbeddingEngine(str(tmp_path), "mps")._load()
    assert attempts == ["mps"]
