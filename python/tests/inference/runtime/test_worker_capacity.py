from __future__ import annotations

import asyncio
import json
from typing import Any

import pytest

from knovum_inference.errors import RpcError
from knovum_inference.runtime import worker
from knovum_inference.transport.protocol import Request


def _frame(request_id: str) -> bytes:
    return json.dumps(
        {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "runtime.health",
            "params": {"sessionId": "session"},
        }
    ).encode()


def test_handler_limit_rejects_excess_requests_but_keeps_fast_notifications_responsive(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    async def scenario() -> None:
        release = asyncio.Event()
        notification_seen = asyncio.Event()
        handled: list[str | int | None] = []
        frames = [_frame(f"pending-{index}") for index in range(worker.MAX_HANDLER_TASKS)]
        frames.extend(
            [
                _frame("overflow"),
                b'{"jsonrpc":"2.0","method":"runtime.cancel","params":{}}',
            ]
        )

        class ControlledInput:
            closed = False

            def __init__(self, loop: asyncio.AbstractEventLoop) -> None:
                pass

            def start(self) -> None:
                pass

            async def receive(self) -> bytes | None:
                if frames:
                    return frames.pop(0)
                await notification_seen.wait()
                release.set()
                return None

            def close(self) -> None:
                self.closed = True

        class CapturingWriter:
            def __init__(self) -> None:
                self.errors: list[tuple[str | int | None, RpcError]] = []

            async def send_response(self, request_id: str | int, response: dict[str, Any]) -> None:
                raise AssertionError("blocked handlers cannot publish responses")

            async def error(self, request_id: str | int | None, error: RpcError) -> None:
                self.errors.append((request_id, error))

        writer = CapturingWriter()
        monkeypatch.setattr(worker, "InputStream", ControlledInput)
        monkeypatch.setattr(worker, "ProtocolWriter", lambda: writer)
        server = worker.Worker()

        async def handle(request: Request) -> None:
            handled.append(request.request_id)
            if request.request_id is None:
                notification_seen.set()
                return
            await release.wait()

        server._handle = handle  # type: ignore[method-assign]
        await asyncio.wait_for(server.serve(), timeout=2)

        assert len(handled) == worker.MAX_HANDLER_TASKS + 1
        assert handled[-1] is None
        assert "overflow" not in handled
        assert len(writer.errors) == 1
        request_id, error = writer.errors[0]
        assert request_id == "overflow"
        assert error.stable_code == "resource_exhausted"

    asyncio.run(scenario())
