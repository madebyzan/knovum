from __future__ import annotations

import hashlib
import os
import re
import stat
import threading
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import onnxruntime as ort

from knovum_inference.errors import RpcError, invalid_params, server_error

MODEL_RESOLUTION = 320
INPUT_SHAPE = (1, 3, MODEL_RESOLUTION, MODEL_RESOLUTION)
MASK_SHAPE = (1, 1, MODEL_RESOLUTION, MODEL_RESOLUTION)
FLOAT32_BYTES = 4
INPUT_BYTES = np.prod(INPUT_SHAPE, dtype=np.int64).item() * FLOAT32_BYTES
MASK_BYTES = np.prod(MASK_SHAPE, dtype=np.int64).item() * FLOAT32_BYTES
_REFERENCE_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,159}\Z")
_REQUEST_ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9-]{0,127}\Z")


@dataclass(frozen=True, slots=True)
class SegmentedMask:
    file_name: str
    sha256: str
    byte_length: int = MASK_BYTES
    shape: tuple[int, int, int, int] = MASK_SHAPE
    data_type: str = "float32_little_endian"


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
        self,
        tensor_file_name: str,
        tensor_sha256: str,
        request_id: str,
        cancelled: threading.Event,
    ) -> SegmentedMask:
        if not _REQUEST_ID.fullmatch(request_id):
            raise invalid_params("Image segmentation request identifier is invalid")
        if not re.fullmatch(r"[0-9a-f]{64}", tensor_sha256):
            raise invalid_params("Image tensor digest is invalid")
        tensor_path = self._input_path(tensor_file_name)
        if cancelled.is_set():
            raise server_error("cancelled", "Image segmentation request was cancelled")
        payload = self._read_input(tensor_path)
        if len(payload) != INPUT_BYTES:
            raise invalid_params("Image tensor has invalid byte length")
        if hashlib.sha256(payload).hexdigest() != tensor_sha256:
            raise server_error("invalid_input", "Image tensor digest does not match")
        tensor = np.frombuffer(payload, dtype="<f4").reshape(INPUT_SHAPE)
        if not np.isfinite(tensor).all():
            raise invalid_params("Image tensor contains non-finite values")
        try:
            output = self._session.run([self._output_name], {self._input_name: tensor})[0]
        except Exception as error:
            raise server_error("invalid_output", "Image segmentation inference failed") from error
        if cancelled.is_set():
            raise server_error("cancelled", "Image segmentation request was cancelled")
        mask = np.asarray(output, dtype="<f4")
        if mask.shape != MASK_SHAPE or not np.isfinite(mask).all():
            raise server_error("invalid_output", "Image segmentation mask is invalid")
        file_name = f"mask-{request_id}.f32"
        target = self._work_directory / file_name
        temporary = self._work_directory / f".{file_name}.tmp"
        if target.exists():
            raise server_error("invalid_output", "Image mask target already exists")
        mask_bytes = mask.tobytes(order="C")
        try:
            with temporary.open("xb") as stream:
                stream.write(mask_bytes)
                stream.flush()
                os.fsync(stream.fileno())
            os.replace(temporary, target)
        except OSError as error:
            temporary.unlink(missing_ok=True)
            raise server_error("resource_exhausted", "Image mask cannot be written") from error
        return SegmentedMask(
            file_name=file_name,
            sha256=hashlib.sha256(mask_bytes).hexdigest(),
        )

    def _input_path(self, file_name: str) -> Path:
        if not _REFERENCE_NAME.fullmatch(file_name) or Path(file_name).name != file_name:
            raise invalid_params("Image tensor reference is invalid")
        path = self._work_directory / file_name
        try:
            metadata = path.lstat()
        except OSError as error:
            raise server_error("invalid_input", "Image tensor is missing") from error
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
            raise server_error("invalid_input", "Image tensor must be a regular file")
        if metadata.st_size != INPUT_BYTES:
            raise invalid_params("Image tensor has invalid byte length")
        return path

    @staticmethod
    def _read_input(path: Path) -> bytes:
        try:
            before = path.lstat()
            flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
            descriptor = os.open(path, flags)
            with os.fdopen(descriptor, "rb", closefd=True) as stream:
                opened = os.fstat(stream.fileno())
                if (
                    not stat.S_ISREG(opened.st_mode)
                    or (before.st_dev, before.st_ino) != (opened.st_dev, opened.st_ino)
                    or opened.st_size != INPUT_BYTES
                ):
                    raise server_error("invalid_input", "Image tensor changed while opening")
                return stream.read(INPUT_BYTES + 1)
        except RpcError:
            raise
        except OSError as error:
            raise server_error("invalid_input", "Image tensor cannot be read") from error
