from __future__ import annotations

import threading
from pathlib import Path

import numpy as np
import onnxruntime as ort
from numpy.typing import NDArray

from knovum_inference.errors import invalid_params, server_error

MAX_PIXELS = 16_777_216


class ImageInpaintingEngine:
    def __init__(self, directory: Path, options: ort.SessionOptions) -> None:
        self._session = ort.InferenceSession(
            str(directory / "migan_pipeline_v2.onnx"), sess_options=options, providers=["CPUExecutionProvider"]
        )
        inputs = self._session.get_inputs()
        if [item.name for item in inputs] != ["image", "mask"] or any(item.type != "tensor(uint8)" for item in inputs):
            raise server_error("model_corrupt", "MI-GAN graph interface differs from its specification")

    def inpaint(
        self, image: NDArray[np.uint8], mask: NDArray[np.uint8], cancelled: threading.Event
    ) -> NDArray[np.uint8]:
        height, width = image.shape[2:]
        if height * width > MAX_PIXELS or mask.shape != (1, 1, height, width):
            raise invalid_params("Inpainting image and mask shape differ")
        if not np.any(mask) or not np.all((mask == 0) | (mask == 255)):
            raise invalid_params("Inpainting requires a non-empty binary selected-region mask")
        if cancelled.is_set():
            raise server_error("cancelled", "Inpainting was cancelled")
        # Product mask: 255 selected. Supplier pipeline: 255 known, 0 hole.
        pixels = self._session.run(["result"], {"image": image, "mask": 255 - mask})[0]
        if cancelled.is_set():
            raise server_error("cancelled", "Inpainting was cancelled")
        if pixels.shape != image.shape or pixels.dtype != np.uint8:
            raise server_error("invalid_output", "MI-GAN output does not match its source")
        # Preserve every unselected source byte, independently of the supplier's compositor.
        return np.where(mask == 255, pixels, image).astype(np.uint8)
