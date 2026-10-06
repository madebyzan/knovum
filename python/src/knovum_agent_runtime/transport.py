from __future__ import annotations

import asyncio
import json
import sys
import threading
from collections.abc import Awaitable, Callable
from concurrent.futures import CancelledError as ReadCancelled
from typing import Any, BinaryIO, NoReturn
from uuid import uuid4

from pydantic import BaseModel

MAX_FRAME_BYTES = 32 * 1024 * 1024
MAX_PENDING_REQUESTS = 64
Handler = Callable[[str, Any], Awaitable[Any]]


class HostRejected(RuntimeError):
    def __init__(self, code: int, message: str) -> None:
        super().__init__(message)
        self.code = code


class ProtocolClosed(RuntimeError):
    pass


def wire_value(value: Any) -> Any:
    if isinstance(value, BaseModel):
        return value.model_dump(mode="json", by_alias=True)
    return value


def reject_nonfinite(value: str) -> NoReturn:
    raise ValueError("agent-runtime-nonfinite-json")


class Peer:
    def __init__(self, source: BinaryIO = sys.stdin.buffer, sink: BinaryIO = sys.stdout.buffer) -> None:
        self.source = source
        self.sink = sink
        self._pending: dict[str, asyncio.Future[Any]] = {}
        self._writer = asyncio.Lock()
        self._closed = asyncio.Event()
        self._handlers: set[asyncio.Task[None]] = set()

    async def request(self, method: str, params: BaseModel) -> Any:
        if self._closed.is_set():
            raise ProtocolClosed("agent-runtime-host-disconnected")
        if len(self._pending) >= MAX_PENDING_REQUESTS:
            raise RuntimeError("agent-runtime-request-limit")
        request_id = str(uuid4())
        response = asyncio.get_running_loop().create_future()
        self._pending[request_id] = response
        try:
            await self._write({"jsonrpc": "2.0", "id": request_id, "method": method, "params": wire_value(params)})
            return await response
        finally:
            self._pending.pop(request_id, None)

    async def notify(self, method: str, params: BaseModel) -> None:
        await self._write({"jsonrpc": "2.0", "method": method, "params": wire_value(params)})

    async def run(self, handler: Handler) -> None:
        loop = asyncio.get_running_loop()
        frames: asyncio.Queue[bytes | OSError | None] = asyncio.Queue(maxsize=8)

        def read() -> None:
            # Windows anonymous stdio pipes cannot rely on Unix add_reader. A daemon
            # reader also permits normal shutdown while the host still owns stdin.
            while not self._closed.is_set():
                try:
                    frame: bytes | OSError = self.source.readline(MAX_FRAME_BYTES + 1)
                except OSError as error:
                    frame = error
                if self._closed.is_set():
                    return
                try:
                    asyncio.run_coroutine_threadsafe(frames.put(frame or None), loop).result()
                except ReadCancelled:
                    return
                if isinstance(frame, OSError) or not frame or len(frame) > MAX_FRAME_BYTES:
                    return

        threading.Thread(target=read, name="agent-runtime-stdin", daemon=True).start()
        receive = asyncio.create_task(frames.get())
        closed = asyncio.create_task(self._closed.wait())
        try:
            while True:
                done, _ = await asyncio.wait([receive, closed], return_when=asyncio.FIRST_COMPLETED)
                if closed in done:
                    break
                frame = receive.result()
                if frame is None:
                    break
                if isinstance(frame, OSError):
                    raise ProtocolClosed("agent-runtime-input-failed") from frame
                self._accept(frame, handler)
                receive = asyncio.create_task(frames.get())
        finally:
            receive.cancel()
            closed.cancel()
            self.close()
            handlers = tuple(self._handlers)
            for task in handlers:
                task.cancel()
            await asyncio.gather(*handlers, return_exceptions=True)

    def close(self) -> None:
        self._closed.set()
        for pending in self._pending.values():
            if not pending.done():
                pending.set_exception(ProtocolClosed("agent-runtime-host-disconnected"))

    def _accept(self, frame: bytes, handler: Handler) -> None:
        if len(frame) > MAX_FRAME_BYTES:
            raise ValueError("agent-runtime-frame-limit")
        value = json.loads(frame, parse_constant=reject_nonfinite)
        if not isinstance(value, dict) or value.get("jsonrpc") != "2.0":
            raise ValueError("agent-runtime-invalid-frame")
        if "method" in value:
            if not isinstance(value["method"], str) or len(self._handlers) >= MAX_PENDING_REQUESTS:
                raise ValueError("agent-runtime-invalid-request")
            if "id" in value and not isinstance(value["id"], str):
                raise ValueError("agent-runtime-invalid-request-id")
            task = asyncio.create_task(self._dispatch(value, handler))
            self._handlers.add(task)
            task.add_done_callback(self._handler_done)
            return
        request_id = value.get("id")
        if not isinstance(request_id, str) or request_id not in self._pending:
            raise ValueError("agent-runtime-unexpected-response")
        pending = self._pending[request_id]
        if pending.done():
            raise ValueError("agent-runtime-duplicate-response")
        if "error" in value:
            error = value["error"]
            if (
                not isinstance(error, dict)
                or not isinstance(error.get("code"), int)
                or not isinstance(error.get("message"), str)
            ):
                raise ValueError("agent-runtime-invalid-error")
            pending.set_exception(HostRejected(error["code"], error["message"]))
        elif "result" in value:
            pending.set_result(value["result"])
        else:
            raise ValueError("agent-runtime-response-missing-result")

    async def _dispatch(self, value: dict[str, Any], handler: Handler) -> None:
        request_id = value.get("id")
        try:
            response_value = await handler(value["method"], value.get("params"))
            response = {"jsonrpc": "2.0", "id": request_id, "result": wire_value(response_value)}
        except (asyncio.CancelledError, ProtocolClosed):
            raise
        except HostRejected as error:
            response = {"jsonrpc": "2.0", "id": request_id, "error": {"code": error.code, "message": str(error)}}
        except Exception:
            # Pydantic and provider exception text can contain credentials/input.
            response = {
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {"code": -32603, "message": "agent-runtime-request-failed"},
            }
        if request_id is not None:
            await self._write(response)

    def _handler_done(self, task: asyncio.Task[None]) -> None:
        self._handlers.discard(task)
        if not task.cancelled() and task.exception() is not None:
            self.close()

    async def _write(self, value: dict[str, Any]) -> None:
        frame = json.dumps(value, ensure_ascii=False, separators=(",", ":"), allow_nan=False).encode("utf-8") + b"\n"
        if len(frame) > MAX_FRAME_BYTES:
            raise ValueError("agent-runtime-frame-limit")
        async with self._writer:
            await asyncio.to_thread(self._write_frame, frame)

    def _write_frame(self, frame: bytes) -> None:
        self.sink.write(frame)
        self.sink.flush()
