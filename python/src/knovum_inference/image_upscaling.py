from __future__ import annotations

import threading
from pathlib import Path

import numpy as np
import onnxruntime as ort
from numpy.typing import NDArray

from knovum_inference.errors import invalid_params, server_error

TILE = 128
CONTEXT = 32
CORE = TILE - 2 * CONTEXT
NATIVE_SCALE = 4
MAX_SOURCE_PIXELS = 4_194_304


class ImageUpscalingEngine:
    def __init__(self, directory: Path, variant: str, options: ort.SessionOptions) -> None:
        filename = {
            "realEsrganGeneralX4v3": "real_esrgan_general_x4v3.onnx",
            "realEsrganX4plus": "real_esrgan_x4plus.onnx",
        }[variant]
        self._session = ort.InferenceSession(
            str(directory / filename), sess_options=options, providers=["CPUExecutionProvider"]
        )
        inputs, outputs = self._session.get_inputs(), self._session.get_outputs()
        if (
            len(inputs) != 1
            or inputs[0].name != "image"
            or inputs[0].shape != [1, 3, TILE, TILE]
            or outputs[0].name != "upscaled_image"
            or outputs[0].shape != [1, 3, TILE * 4, TILE * 4]
        ):
            raise server_error("model_corrupt", "Upscaling graph interface differs from its specification")

    def upscale(self, image: NDArray[np.uint8], cancelled: threading.Event) -> NDArray[np.uint8]:
        _, channels, height, width = image.shape
        if channels != 3 or height * width > MAX_SOURCE_PIXELS:
            raise invalid_params("Upscaling source exceeds its pixel budget")
        # Context is discarded, so every output pixel comes from the interior of one inferred tile.
        padded = np.pad(image, ((0, 0), (0, 0), (CONTEXT, TILE), (CONTEXT, TILE)), mode="edge")
        output = np.empty((1, 3, height * 4, width * 4), dtype=np.uint8)
        for top in range(0, height, CORE):
            for left in range(0, width, CORE):
                if cancelled.is_set():
                    raise server_error("cancelled", "Upscaling was cancelled")
                tile = padded[:, :, top : top + TILE, left : left + TILE].astype(np.float32) / 255
                pixels = self._session.run(["upscaled_image"], {"image": tile})[0]
                if pixels.shape != (1, 3, TILE * 4, TILE * 4) or not np.isfinite(pixels).all():
                    raise server_error("invalid_output", "Upscaling produced an invalid tile")
                rows, columns = min(CORE, height - top) * 4, min(CORE, width - left) * 4
                border = CONTEXT * 4
                interior = pixels[:, :, border : border + rows, border : border + columns]
                output[:, :, top * 4 : top * 4 + rows, left * 4 : left * 4 + columns] = np.clip(
                    np.rint(interior * 255), 0, 255
                ).astype(np.uint8)
        return output
