from __future__ import annotations

import threading
from pathlib import Path

import numpy as np
import onnxruntime as ort

from knovum_inference.contracts.generated.image import TensorDataType, TensorFileReference
from knovum_inference.errors import invalid_params, server_error

MODEL_RESOLUTION = 320
INPUT_SHAPE = (1, 3, MODEL_RESOLUTION, MODEL_RESOLUTION)
MASK_SHAPE = (1, 1, MODEL_RESOLUTION, MODEL_RESOLUTION)
FLOAT32_BYTES = 4
INPUT_BYTES = np.prod(INPUT_SHAPE, dtype=np.int64).item() * FLOAT32_BYTES
MASK_BYTES = np.prod(MASK_SHAPE, dtype=np.int64).item() * FLOAT32_BYTES


class ImageSegmentationEngine:
    def __init__(self, model_path: str, work_directory: str, cpu_threads: int) -> None:
        model = Path(model_path)
        work = Path(work_directory)
        if not model.is_absolute() or not model.is_file():
            raise server_error("model_missing", "Image segmentation model is missing")
        if not work.is_absolute() or not work.is_dir():
            raise invalid_params("Work directory must be an existing absolute directory")
        self._work_directory = work.resolve(strict=True)
        options = ort.SessionOptions()
        options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        options.intra_op_num_threads = cpu_threads
        options.inter_op_num_threads = 1
        try:
            self._session = ort.InferenceSession(str(model), sess_options=options, providers=["CPUExecutionProvider"])
        except Exception as error:
            raise server_error("model_corrupt", "Image segmentation model cannot be loaded") from error
        inputs = self._session.get_inputs()
        outputs = self._session.get_outputs()
        if len(inputs) != 1 or not outputs:
            raise server_error("model_corrupt", "Image segmentation model interface is invalid")
        self._input_name = inputs[0].name
        self._output_name = outputs[0].name

    def segment(
        self, tensor_file_name: str, tensor_sha256: str, request_id: str, cancelled: threading.Event
    ) -> TensorFileReference:
        from knovum_inference.image_tensors import read_tensor, write_tensor

        reference = TensorFileReference.model_validate(
            {
                "fileName": tensor_file_name,
                "byteLength": INPUT_BYTES,
                "sha256": tensor_sha256,
                "shape": list(INPUT_SHAPE),
                "dataType": TensorDataType.float32_little_endian,
            }
        )
        if cancelled.is_set():
            raise server_error("cancelled", "Image segmentation request was cancelled")
        tensor = read_tensor(self._work_directory, reference, INPUT_BYTES)
        try:
            output = self._session.run([self._output_name], {self._input_name: tensor})[0]
        except Exception as error:
            raise server_error("invalid_output", "Image segmentation inference failed") from error
        if cancelled.is_set():
            raise server_error("cancelled", "Image segmentation request was cancelled")
        mask = np.asarray(output, dtype="<f4")
        if mask.shape != MASK_SHAPE or not np.isfinite(mask).all():
            raise server_error("invalid_output", "Image segmentation mask is invalid")
        return write_tensor(self._work_directory, mask, request_id, "mask")
