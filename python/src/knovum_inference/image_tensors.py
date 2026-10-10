from __future__ import annotations

import hashlib
import os
import re
import stat
from pathlib import Path
from typing import Literal

import numpy as np
from numpy.typing import NDArray

from knovum_inference.contracts.generated.image import TensorDataType, TensorFileReference
from knovum_inference.errors import RpcError, invalid_params, server_error

_REFERENCE_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,159}\Z")
_REQUEST_ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9-]{0,127}\Z")


def read_tensor(directory: Path, reference: TensorFileReference, maximum_bytes: int) -> NDArray[np.generic]:
    shape = tuple(item.root for item in reference.shape)
    dtype = np.dtype("<f4" if reference.data_type is TensorDataType.float32_little_endian else "u1")
    required = int(np.prod(shape, dtype=np.int64)) * dtype.itemsize
    if not all(0 < value <= 16384 for value in shape) or not 0 < required <= maximum_bytes:
        raise invalid_params("Image tensor exceeds the admitted shape or byte budget")
    if reference.byte_length != required or not _REFERENCE_NAME.fullmatch(reference.file_name):
        raise invalid_params("Image tensor reference is invalid")
    if not re.fullmatch(r"[0-9a-f]{64}", reference.sha256):
        raise invalid_params("Image tensor digest is invalid")
    path = directory / reference.file_name
    try:
        before = path.lstat()
        if not stat.S_ISREG(before.st_mode) or before.st_size != required:
            raise invalid_params("Image tensor must be a regular file with the admitted byte length")
        descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        with os.fdopen(descriptor, "rb", closefd=True) as stream:
            opened = os.fstat(stream.fileno())
            if (before.st_dev, before.st_ino) != (opened.st_dev, opened.st_ino) or opened.st_size != required:
                raise invalid_params("Image tensor changed while opening")
            payload = stream.read(required + 1)
    except RpcError:
        raise
    except OSError as error:
        raise server_error("invalid_input", "Image tensor cannot be read") from error
    if len(payload) != required or hashlib.sha256(payload).hexdigest() != reference.sha256:
        raise server_error("invalid_input", "Image tensor length or digest differs")
    tensor = np.frombuffer(payload, dtype=dtype).reshape(shape)
    if not np.isfinite(tensor).all():
        raise invalid_params("Image tensor contains non-finite values")
    return tensor


def write_tensor(
    directory: Path, tensor: NDArray[np.generic], request_id: str, purpose: Literal["image", "mask"] = "image"
) -> TensorFileReference:
    if not _REQUEST_ID.fullmatch(request_id):
        raise invalid_params("Image request identifier is invalid")
    if tensor.ndim != 4 or not np.isfinite(tensor).all():
        raise server_error("invalid_output", "Image output tensor is invalid")
    if (
        tensor.dtype not in (np.dtype("uint8"), np.dtype("float32"))
        or tensor.nbytes > 256 * 1024 * 1024
        or any(not 0 < size <= 16384 for size in tensor.shape)
    ):
        raise server_error("invalid_output", "Image output exceeds its type or byte budget")
    data_type = TensorDataType.uint8 if tensor.dtype == np.uint8 else TensorDataType.float32_little_endian
    payload = tensor.astype("u1" if data_type is TensorDataType.uint8 else "<f4", copy=False).tobytes(order="C")
    name = f"mask-{request_id}.f32" if purpose == "mask" else f"image-{request_id}.tensor"
    temporary = directory / f".{name}.part"
    target = directory / name
    if target.exists():
        raise server_error("invalid_output", "Image output reference already exists")
    try:
        with temporary.open("xb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, target)
    except OSError as error:
        temporary.unlink(missing_ok=True)
        raise server_error("resource_exhausted", "Image tensor cannot be written") from error
    return TensorFileReference.model_validate(
        {
            "fileName": name,
            "byteLength": len(payload),
            "sha256": hashlib.sha256(payload).hexdigest(),
            "shape": list(tensor.shape),
            "dataType": data_type,
        }
    )
