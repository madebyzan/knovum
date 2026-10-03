from __future__ import annotations

import asyncio
import sys
import threading
from contextlib import suppress

from knovum_inference.protocol import MAX_MESSAGE_BYTES

OVERSIZED = object()


class InputStream:
    def __init__(self, loop: asyncio.AbstractEventLoop) -> None:
        self._loop = loop
        self._queue: asyncio.Queue[bytes | object | None] = asyncio.Queue()
        self._thread = threading.Thread(target=self._read, daemon=True)

    def start(self) -> None:
        self._thread.start()

    async def receive(self) -> bytes | object | None:
        return await self._queue.get()

    def _read(self) -> None:
        while True:
            line = sys.stdin.buffer.readline(MAX_MESSAGE_BYTES + 2)
            if not line:
                self._deliver(None)
                return
            complete = line.endswith(b"\n")
            payload = line[:-1]
            if payload.endswith(b"\r"):
                payload = payload[:-1]
            if len(payload) > MAX_MESSAGE_BYTES or not complete:
                if not complete:
                    self._drain_line()
                self._deliver(OVERSIZED)
            else:
                self._deliver(payload)

    def _drain_line(self) -> None:
        while True:
            chunk = sys.stdin.buffer.readline(MAX_MESSAGE_BYTES + 2)
            if not chunk or chunk.endswith(b"\n"):
                return

    def _deliver(self, value: bytes | object | None) -> None:
        with suppress(RuntimeError):
            self._loop.call_soon_threadsafe(self._queue.put_nowait, value)
