from __future__ import annotations

import threading
from pathlib import Path

import numpy as np
import onnxruntime as ort
from numpy.typing import NDArray

from knovum_inference.errors import invalid_params, server_error


class ImageDepthEngine:
    def __init__(self, directory: Path, variant: str, options: ort.SessionOptions) -> None:
        filename = "model_uint8.onnx" if variant == "depthAnythingV2SmallUint8" else "model.onnx"
        self._session = ort.InferenceSession(
            str(directory / filename), sess_options=options, providers=["CPUExecutionProvider"]
        )
        if [item.name for item in self._session.get_inputs()] != ["pixel_values"]:
            raise server_error("model_corrupt", "Depth graph interface differs from its specification")

    def estimate(self, normalized_rgb: NDArray[np.float32], cancelled: threading.Event) -> NDArray[np.float32]:
        _, channels, height, width = normalized_rgb.shape
        if channels != 3 or height % 14 or width % 14 or height * width > 1_048_576:
            raise invalid_params("Depth tensor exceeds the fixed processor budget")
        if cancelled.is_set():
            raise server_error("cancelled", "Depth estimation was cancelled")
        depth = np.asarray(
            self._session.run(["predicted_depth"], {"pixel_values": normalized_rgb})[0], dtype=np.float32
        )
        if cancelled.is_set():
            raise server_error("cancelled", "Depth estimation was cancelled")
        if depth.shape != (1, height, width) or not np.isfinite(depth).all():
            raise server_error("invalid_output", "Depth estimation produced an invalid relative field")
        low, high = float(depth.min()), float(depth.max())
        if high <= low:
            raise server_error("invalid_output", "Depth estimation produced a constant field")
        return ((depth - low) / (high - low)).astype(np.float32)[:, np.newaxis, :, :]
