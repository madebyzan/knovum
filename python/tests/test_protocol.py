import pytest

from knovum_inference.errors import RpcError
from knovum_inference.protocol import parse_request


def test_request_requires_json_rpc_identity() -> None:
    request = parse_request(b'{"jsonrpc":"2.0","id":"request-1","method":"runtime.health","params":{}}')
    assert request.request_id == "request-1"
    assert request.method == "runtime.health"


@pytest.mark.parametrize(
    "line",
    [
        b"not json",
        b'{"jsonrpc":"1.0","id":1,"method":"runtime.health"}',
        b'{"jsonrpc":"2.0","id":true,"method":"runtime.health"}',
        b'{"jsonrpc":"2.0","method":"embedding.encode","params":{}}',
    ],
)
def test_invalid_request_is_rejected(line: bytes) -> None:
    with pytest.raises(RpcError):
        parse_request(line)


@pytest.mark.parametrize("method", ["runtime.cancel", "runtime.ackChunk"])
def test_runtime_control_may_be_notification(method: str) -> None:
    request = parse_request(f'{{"jsonrpc":"2.0","method":"{method}","params":{{}}}}'.encode())
    assert request.request_id is None
