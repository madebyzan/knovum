from __future__ import annotations

import asyncio
import json
import sys
from dataclasses import dataclass
from typing import Any

from knovum_inference.errors import RpcError

MAX_MESSAGE_BYTES = 1_048_576


@dataclass(frozen=True, slots=True)
class Request:
    request_id: str | int | None
    method: str
    params: dict[str, Any]


def parse_request(line: bytes) -> Request:
    try:
        value = json.loads(line)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RpcError(-32700, "Invalid JSON", "invalid_output") from error
    if not isinstance(value, dict) or value.get("jsonrpc") != "2.0":
        raise RpcError(-32600, "Invalid JSON-RPC request", "invalid_output")
    method = value.get("method")
    params = value.get("params", {})
    if not isinstance(method, str) or not isinstance(params, dict):
        raise RpcError(-32600, "Invalid JSON-RPC method or params", "invalid_output")
    if "id" not in value:
        if method != "runtime.cancel":
            raise RpcError(-32600, "Only runtime.cancel may be a notification", "invalid_output")
        request_id = None
    else:
        request_id = value["id"]
        if isinstance(request_id, bool) or not isinstance(request_id, (str, int)):
            raise RpcError(-32600, "Request id must be a string or integer", "invalid_output")
    return Request(request_id, method, params)


class ProtocolWriter:
    def __init__(self) -> None:
        self._lock = asyncio.Lock()

    async def result(self, request_id: str | int, result: dict[str, Any]) -> None:
        value = {"jsonrpc": "2.0", "id": request_id, "result": result}
        encoded = self._encode(value)
        if len(encoded) >= MAX_MESSAGE_BYTES:
            await self.error(
                request_id,
                RpcError(-32000, "Response exceeds 1 MiB", "resource_exhausted"),
            )
            return
        await self._write(encoded)

    async def error(self, request_id: str | int | None, error: RpcError) -> None:
        await self._write(self._encode({"jsonrpc": "2.0", "id": request_id, "error": error.as_object()}))

    async def notification(self, method: str, params: dict[str, Any]) -> None:
        await self._write(self._encode({"jsonrpc": "2.0", "method": method, "params": params}))

    @staticmethod
    def _encode(value: dict[str, Any]) -> bytes:
        return json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(",", ":")).encode()

    async def _write(self, encoded: bytes) -> None:
        if len(encoded) >= MAX_MESSAGE_BYTES:
            raise RuntimeError("Protocol output exceeds 1 MiB")
        async with self._lock:
            sys.stdout.buffer.write(encoded + b"\n")
            sys.stdout.buffer.flush()
