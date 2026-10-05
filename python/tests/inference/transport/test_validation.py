from __future__ import annotations

from typing import Any

import pytest

from knovum_inference.contracts.generated.control import RuntimeHealthObservation
from knovum_inference.contracts.generated.image import ImageSegmentation
from knovum_inference.errors import RpcError
from knovum_inference.transport.validation import validate_response


def mask_result(data_type: str) -> dict[str, Any]:
    return {
        "requestId": "image-request",
        "attemptEpoch": 1,
        "modelSpecId": "u2net",
        "maskRef": {
            "fileName": "mask.f32",
            "byteLength": 16,
            "sha256": "a" * 64,
            "shape": [1, 1, 2, 2],
            "dataType": data_type,
        },
    }


def test_result_accepts_the_generated_json_enum_contract() -> None:
    value = mask_result("float32_little_endian")
    assert validate_response(ImageSegmentation, value) == value


def test_result_rejects_unknown_enum_values() -> None:
    with pytest.raises(RpcError) as captured:
        validate_response(ImageSegmentation, mask_result("float64_little_endian"))
    assert captured.value.stable_code == "invalid_output"


@pytest.mark.parametrize("resident_bytes", ["1024", True, -1, float("nan"), float("inf")])
def test_result_preserves_strict_numeric_validation(resident_bytes: Any) -> None:
    with pytest.raises(RpcError) as captured:
        validate_response(RuntimeHealthObservation, {"sessionId": "session", "residentBytes": resident_bytes})
    assert captured.value.stable_code == "invalid_output"
