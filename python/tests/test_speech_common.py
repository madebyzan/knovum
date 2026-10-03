from pathlib import Path
from uuid import uuid4

import pytest

from knovum_inference.errors import RpcError
from knovum_inference.speech_common import attempt_directory, milliseconds


def test_attempt_directory_is_private_and_unique(tmp_path: Path) -> None:
    request_id = str(uuid4())
    path = attempt_directory(tmp_path, request_id)
    assert path.parent == tmp_path
    assert path.name == f"attempt-{request_id}"
    with pytest.raises(RpcError):
        attempt_directory(tmp_path, request_id)


@pytest.mark.parametrize("request_id", ["../outside", "not-a-uuid", "00000000-0000-0000-0000-000000000000/"])
def test_attempt_directory_rejects_unsafe_identity(tmp_path: Path, request_id: str) -> None:
    with pytest.raises(RpcError):
        attempt_directory(tmp_path, request_id)


@pytest.mark.parametrize("seconds", [float("nan"), float("inf"), -1.0])
def test_invalid_timestamp_is_rejected(seconds: float) -> None:
    with pytest.raises(RpcError):
        milliseconds(seconds)
