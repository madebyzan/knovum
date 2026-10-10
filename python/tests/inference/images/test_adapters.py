from __future__ import annotations

import importlib
import sys
import threading
from types import SimpleNamespace
from typing import Any

import numpy as np
import pytest

from knovum_inference.contracts.generated.image import ImageSelectionPoint, ImageSelectionRect
from knovum_inference.errors import RpcError


@pytest.fixture
def engines(monkeypatch: pytest.MonkeyPatch) -> dict[str, Any]:
    monkeypatch.setitem(sys.modules, "onnxruntime", SimpleNamespace())
    return {
        name: importlib.import_module(f"knovum_inference.image_{name}")
        for name in ["upscaling", "inpainting", "selection", "depth"]
    }


def test_upscale_tile_interiors_cover_non_square_source_without_seams(engines: dict[str, Any]) -> None:
    class NearestGraph:
        def run(self, names: list[str], inputs: dict[str, Any]) -> list[Any]:
            return [np.repeat(np.repeat(inputs["image"], 4, axis=2), 4, axis=3)]

    engine = engines["upscaling"].ImageUpscalingEngine.__new__(engines["upscaling"].ImageUpscalingEngine)
    engine._session = NearestGraph()
    image = np.arange(3 * 70 * 90, dtype=np.uint8).reshape(1, 3, 70, 90)
    pixels = engine.upscale(image, threading.Event())
    np.testing.assert_array_equal(pixels, np.repeat(np.repeat(image, 4, axis=2), 4, axis=3))
    token = threading.Event()
    token.set()
    with pytest.raises(RpcError, match="cancelled"):
        engine.upscale(image, token)


def test_inpainting_flips_supplier_polarity_and_protects_unselected_bytes(engines: dict[str, Any]) -> None:
    class InpaintingGraph:
        def run(self, names: list[str], inputs: dict[str, Any]) -> list[Any]:
            assert inputs["mask"][0, 0, 1, 2] == 0
            return [np.full_like(inputs["image"], 255)]

    engine = engines["inpainting"].ImageInpaintingEngine.__new__(engines["inpainting"].ImageInpaintingEngine)
    engine._session = InpaintingGraph()
    image = np.arange(3 * 3 * 4, dtype=np.uint8).reshape(1, 3, 3, 4)
    mask = np.zeros((1, 1, 3, 4), dtype=np.uint8)
    mask[0, 0, 1, 2] = 255
    pixels = engine.inpaint(image, mask, threading.Event())
    np.testing.assert_array_equal(pixels, np.where(mask == 255, 255, image))
    mask[0, 0, 0, 0] = 1
    with pytest.raises(RpcError):
        engine.inpaint(image, mask, threading.Event())


def test_selection_reuses_only_the_current_encoding_and_supports_box_prompts(engines: dict[str, Any]) -> None:
    class Encoder:
        calls = 0

        def run(self, names: list[str], inputs: dict[str, Any]) -> list[Any]:
            self.calls += 1
            return [np.zeros((1, 256, 64, 64), dtype=np.float32)]

    class Decoder:
        def run(self, names: list[str], inputs: dict[str, Any]) -> list[Any]:
            assert inputs["batched_point_labels"].tolist() == [[[1, 2, 3]]]
            masks = np.zeros((1, 1, 3, 8, 10), dtype=np.float32)
            masks[:, :, 1] = 1
            return [masks, np.asarray([[[0.1, 0.9, 0.2]]], dtype=np.float32)]

    engine = engines["selection"].ImageSelectionEngine.__new__(engines["selection"].ImageSelectionEngine)
    engine._encoder, engine._decoder, engine._encoding = Encoder(), Decoder(), None
    image = np.zeros((1, 3, 8, 10), dtype=np.uint8)
    point = ImageSelectionPoint(x=0.5, y=0.5, foreground=True)
    rect = ImageSelectionRect(x=0.1, y=0.1, width=0.8, height=0.8)
    for identity in ["source-one", "source-one", "source-two"]:
        selected = engine.select(image, identity, [point], rect, threading.Event())
        assert np.all(selected == 255)
    assert engine._encoder.calls == 2


def test_depth_rejects_constant_or_nonfinite_fields(engines: dict[str, Any]) -> None:
    class DepthGraph:
        def __init__(self, field: Any) -> None:
            self.field = field

        def run(self, names: list[str], inputs: dict[str, Any]) -> list[Any]:
            return [self.field]

    engine = engines["depth"].ImageDepthEngine.__new__(engines["depth"].ImageDepthEngine)
    image = np.zeros((1, 3, 14, 14), dtype=np.float32)
    for value in [0, float("nan"), float("inf")]:
        engine._session = DepthGraph(np.full((1, 14, 14), value, dtype=np.float32))
        with pytest.raises(RpcError):
            engine.estimate(image, threading.Event())
    engine._session = DepthGraph(np.arange(196, dtype=np.float32).reshape(1, 14, 14))
    field = engine.estimate(image, threading.Event())
    assert field.shape == (1, 1, 14, 14)
    assert field.min() == 0 and field.max() == 1
