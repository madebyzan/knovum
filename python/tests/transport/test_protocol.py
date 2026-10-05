import pytest

from knovum_inference.errors import RpcError
from knovum_inference.transport.protocol import parse_request


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


@pytest.mark.parametrize("encoding", ["utf-16", "utf-32"])
def test_protocol_rejects_non_utf8_json(encoding: str) -> None:
    line = '{"jsonrpc":"2.0","id":1,"method":"runtime.health"}'.encode(encoding)
    with pytest.raises(RpcError) as captured:
        parse_request(line)
    assert captured.value.code == -32700


@pytest.mark.parametrize("number", ["NaN", "Infinity", "-Infinity"])
def test_protocol_rejects_non_json_numeric_literals(number: str) -> None:
    line = f'{{"jsonrpc":"2.0","id":1,"method":"runtime.health","params":{{"value":{number}}}}}'.encode()
    with pytest.raises(RpcError) as captured:
        parse_request(line)
    assert captured.value.code == -32700


def test_excessive_json_nesting_is_a_parse_error() -> None:
    with pytest.raises(RpcError) as captured:
        parse_request(b"[" * 2000 + b"0" + b"]" * 2000)
    assert captured.value.code == -32700
