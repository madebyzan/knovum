from __future__ import annotations

import asyncio
import sys
import threading
from concurrent.futures import CancelledError, Future

from knovum_inference.transport.protocol import MAX_MESSAGE_BYTES

OVERSIZED = object()
MAX_BUFFERED_MESSAGES = 8


class InputStream:
    def __init__(self, loop: asyncio.AbstractEventLoop) -> None:
        self._loop = loop
        self._queue: asyncio.Queue[bytes | object | None] = asyncio.Queue(maxsize=MAX_BUFFERED_MESSAGES)
        self._closed = threading.Event()
        self._delivery_lock = threading.Lock()
        self._delivery: Future[None] | None = None
        self._thread = threading.Thread(target=self._read, name="knovum-inference-stdin", daemon=True)

    def start(self) -> None:
        self._thread.start()

    async def receive(self) -> bytes | object | None:
        return await self._queue.get()

    def close(self) -> None:
        self._closed.set()
        with self._delivery_lock:
            if self._delivery is not None:
                self._delivery.cancel()

    def _read(self) -> None:
        while not self._closed.is_set():
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
                if not self._deliver(OVERSIZED):
                    return
            else:
                if not self._deliver(payload):
                    return

    def _drain_line(self) -> None:
        while True:
            chunk = sys.stdin.buffer.readline(MAX_MESSAGE_BYTES + 2)
            if not chunk or chunk.endswith(b"\n"):
                return

    def _deliver(self, value: bytes | object | None) -> bool:
        if self._closed.is_set():
            return False
        enqueue = self._queue.put(value)
        try:
            delivery = asyncio.run_coroutine_threadsafe(enqueue, self._loop)
        except RuntimeError:
            enqueue.close()
            return False
        with self._delivery_lock:
            if self._closed.is_set():
                delivery.cancel()
                return False
            self._delivery = delivery
        try:
            delivery.result()
        except CancelledError:
            return False
        finally:
            with self._delivery_lock:
                if self._delivery is delivery:
                    self._delivery = None
        return not self._closed.is_set()
