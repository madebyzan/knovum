from __future__ import annotations

import threading
from pathlib import Path

import numpy as np
import onnxruntime as ort
from numpy.typing import NDArray

from knovum_inference.contracts.generated.image import ImageSelectionPoint, ImageSelectionRect
from knovum_inference.errors import invalid_params, server_error


class ImageSelectionEngine:
    def __init__(self, directory: Path, variant: str, options: ort.SessionOptions) -> None:
        name = "ti" if variant == "efficientSamTi" else "s"
        self._encoder = ort.InferenceSession(
            str(directory / f"efficientsam_{name}_encoder.onnx"),
            sess_options=options,
            providers=["CPUExecutionProvider"],
        )
        self._decoder = ort.InferenceSession(
            str(directory / f"efficientsam_{name}_decoder.onnx"),
            sess_options=options,
            providers=["CPUExecutionProvider"],
        )
        if [item.name for item in self._encoder.get_inputs()] != ["batched_images"] or [
            item.name for item in self._decoder.get_inputs()
        ] != ["image_embeddings", "batched_point_coords", "batched_point_labels", "orig_im_size"]:
            raise server_error("model_corrupt", "Selection graph interface differs from its specification")
        # One encoding, owned by this verified worker; no persistent or cross-session embedding cache.
        self._encoding: tuple[str, NDArray[np.float32]] | None = None

    def select(
        self,
        image: NDArray[np.uint8],
        input_identity: str,
        points: list[ImageSelectionPoint],
        box_rect: ImageSelectionRect | None,
        cancelled: threading.Event,
    ) -> NDArray[np.uint8]:
        height, width = image.shape[2:]
        if (
            height * width > 4_194_304
            or not 1 <= len(points) + (2 if box_rect else 0) <= 6
            or (box_rect is None and not any(p.foreground for p in points))
        ):
            raise invalid_params("Selection requires one to six points and a bounded source image")
        if any(not np.isfinite([p.x, p.y]).all() or not 0 <= p.x <= 1 or not 0 <= p.y <= 1 for p in points):
            raise invalid_params("Selection point lies outside the rendered image")
        if cancelled.is_set():
            raise server_error("cancelled", "Selection was cancelled")
        if self._encoding is None or self._encoding[0] != input_identity:
            embeddings = self._encoder.run(["image_embeddings"], {"batched_images": image.astype(np.float32) / 255})[0]
            if embeddings.shape != (1, 256, 64, 64) or not np.isfinite(embeddings).all():
                raise server_error("invalid_output", "Selection encoding is invalid")
            self._encoding = (input_identity, embeddings)
        coords = [[p.x * width, p.y * height] for p in points]
        prompt_labels = [1 if p.foreground else 0 for p in points]
        if box_rect is not None:
            x, y, w, h = box_rect.x, box_rect.y, box_rect.width, box_rect.height
            if not np.isfinite([x, y, w, h]).all() or x < 0 or y < 0 or w <= 0 or h <= 0 or x + w > 1 or y + h > 1:
                raise invalid_params("Selection box lies outside the rendered image")
            coords.extend([[x * width, y * height], [(x + w) * width, (y + h) * height]])
            prompt_labels.extend([2, 3])
        coordinates = np.asarray([[coords]], dtype=np.float32)
        labels = np.asarray([[prompt_labels]], dtype=np.float32)
        masks_output, scores_output = self._decoder.run(
            ["output_masks", "iou_predictions"],
            {
                "image_embeddings": self._encoding[1],
                "batched_point_coords": coordinates,
                "batched_point_labels": labels,
                "orig_im_size": np.asarray([height, width], dtype=np.int64),
            },
        )
        masks = np.asarray(masks_output)
        scores = np.asarray(scores_output)
        if cancelled.is_set():
            raise server_error("cancelled", "Selection was cancelled")
        if (
            masks.shape != (1, 1, 3, height, width)
            or scores.shape != (1, 1, 3)
            or not (np.isfinite(masks).all() and np.isfinite(scores).all())
        ):
            raise server_error("invalid_output", "Selection mask is invalid")
        best = int(np.argmax(scores[0, 0]))
        return (masks[:, :, best] > 0).astype(np.uint8) * 255
