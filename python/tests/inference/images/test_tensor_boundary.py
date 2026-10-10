from __future__ import annotations

import hashlib
from pathlib import Path

import numpy as np
import pytest

from knovum_inference.contracts.generated.image import TensorDataType, TensorFileReference
from knovum_inference.errors import RpcError
from knovum_inference.image_tensors import read_tensor, write_tensor


def reference(name: str, payload: bytes, dtype: TensorDataType = TensorDataType.uint8) -> TensorFileReference:
    return TensorFileReference.model_validate(
        {
            "fileName": name,
            "byteLength": len(payload),
            "sha256": hashlib.sha256(payload).hexdigest(),
            "shape": [1, 3, 1, 2],
            "dataType": dtype,
        }
    )


def test_round_trip_preserves_rgb_bytes(tmp_path: Path) -> None:
    pixels = np.arange(6, dtype=np.uint8).reshape(1, 3, 1, 2)
    output = write_tensor(tmp_path, pixels, "request")
    np.testing.assert_array_equal(read_tensor(tmp_path, output, 6), pixels)
    with pytest.raises(RpcError):
        write_tensor(tmp_path, pixels, "request")


@pytest.mark.parametrize("mutation", ["digest", "oversize", "symlink", "traversal"])
def test_untrusted_tensor_references_are_rejected(tmp_path: Path, mutation: str) -> None:
    payload = bytes(range(6))
    path = tmp_path / "source.tensor"
    path.write_bytes(payload)
    declared = reference(path.name, payload)
    maximum = 6
    if mutation == "digest":
        path.write_bytes(b"x" * 6)
    elif mutation == "oversize":
        maximum = 5
    elif mutation == "symlink":
        link = tmp_path / "link.tensor"
        link.symlink_to(path)
        declared = reference(link.name, payload)
    else:
        declared = reference("../source.tensor", payload)
    with pytest.raises(RpcError):
        read_tensor(tmp_path, declared, maximum)


def test_nonfinite_input_and_invalid_output_type_are_rejected(tmp_path: Path) -> None:
    payload = np.full((1, 3, 1, 2), np.nan, dtype="<f4").tobytes()
    (tmp_path / "source.tensor").write_bytes(payload)
    with pytest.raises(RpcError):
        read_tensor(tmp_path, reference("source.tensor", payload, TensorDataType.float32_little_endian), 24)
    with pytest.raises(RpcError):
        write_tensor(tmp_path, np.ones((1, 3, 1, 2), dtype=np.float64), "request")
