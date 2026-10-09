from __future__ import annotations

import asyncio
import json
import threading
from pathlib import Path
from typing import Any, cast

import pytest

from knovum_inference import PROTOCOL_VERSION
from knovum_inference.contracts.generated.control import RuntimeInitializeParams
from knovum_inference.errors import RpcError
from knovum_inference.runtime import session, worker
from knovum_inference.runtime.session import Session
from knovum_inference.transport.protocol import Request
from knovum_inference.transport.validation import validate_params


def initialization(directory: Path, threads: int = 4) -> Request:
    return Request(
        "initialize",
        "runtime.initialize",
        {
            "protocolVersion": PROTOCOL_VERSION,
            "runtimeManifestPath": str(directory / "runtime-manifest.json"),
            "runtimeDigest": "a" * 64,
            "sessionId": "session",
            "capability": "embedding",
            "capabilitySpecId": "embedding-spec",
            "deviceProfile": {"device": "cpu", "executionProfileId": "cpu-f32-threads4-v1", "cpuThreads": threads},
            "workDirectory": str(directory),
            "modelPaths": {"embedding": str(directory)},
        },
    )


class CapturingWriter:
    def __init__(self) -> None:
        self.results: list[dict[str, Any]] = []
        self.errors: list[RpcError] = []

    async def send_response(self, request_id: str | int, result: dict[str, Any]) -> None:
        self.results.append(result)

    async def error(self, request_id: str | int | None, error: RpcError) -> None:
        self.errors.append(error)


def blocked_preparation(monkeypatch: pytest.MonkeyPatch) -> tuple[threading.Event, threading.Event]:
    entered, release = threading.Event(), threading.Event()

    def prepare(params: Any) -> Session:
        entered.set()
        assert release.wait(5), "test did not release initialization"
        return Session(
            params.session_id,
            params.capability,
            params.capability_spec_id,
            params.device_profile,
            cast(Any, object()),
            Path(params.work_directory),
        )

    monkeypatch.setattr(worker, "prepare_session", prepare)
    return entered, release


def test_pending_initialization_keeps_controls_responsive_and_rejects_duplicates(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    entered, release = blocked_preparation(monkeypatch)

    async def scenario() -> None:
        server = worker.Worker()
        request = initialization(tmp_path)
        preparing = asyncio.create_task(server._dispatch(request))
        try:
            assert await asyncio.wait_for(asyncio.to_thread(entered.wait, 1), timeout=2)
            controls = [
                Request("health", "runtime.health", {"sessionId": "session"}),
                Request(None, "runtime.cancel", {"targetRequestId": "work", "attemptEpoch": 1}),
                request,
            ]
            for control in controls:
                with pytest.raises(RpcError) as captured:
                    await asyncio.wait_for(server._dispatch(control), timeout=0.5)
                assert captured.value.stable_code == "runtime_mismatch"
        finally:
            release.set()
        readiness = await preparing
        assert readiness["sessionId"] == "session"
        assert readiness["capabilities"] == ["embedding.encode"]

    asyncio.run(scenario())


def test_transport_close_cannot_activate_a_pending_session(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    entered, release = blocked_preparation(monkeypatch)
    request = initialization(tmp_path)
    line = json.dumps({"jsonrpc": "2.0", "id": request.request_id, "method": request.method, "params": request.params})

    async def scenario() -> None:
        closed = asyncio.Event()
        writer = CapturingWriter()

        class ClosingInput:
            def __init__(self, loop: asyncio.AbstractEventLoop) -> None:
                self.first = True

            def start(self) -> None:
                pass

            def close(self) -> None:
                pass

            async def receive(self) -> bytes | None:
                if self.first:
                    self.first = False
                    return line.encode()
                assert await asyncio.to_thread(entered.wait, 1)
                closed.set()
                return None

        monkeypatch.setattr(worker, "InputStream", ClosingInput)
        monkeypatch.setattr(worker, "ProtocolWriter", lambda: writer)
        server = worker.Worker()
        serving = asyncio.create_task(server.serve())
        try:
            await asyncio.wait_for(closed.wait(), timeout=2)
            await asyncio.wait_for(server._terminate.wait(), timeout=2)
        finally:
            release.set()
        await asyncio.wait_for(serving, timeout=2)
        assert writer.results == []
        assert [error.stable_code for error in writer.errors] == ["cancelled"]
        with pytest.raises(RpcError) as captured:
            await server._dispatch(Request("health", "runtime.health", {"sessionId": "session"}))
        assert captured.value.stable_code == "runtime_mismatch"

    asyncio.run(scenario())


def test_failed_preparation_never_exposes_a_ready_session(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    def fail(params: Any) -> Session:
        raise ImportError("compiled module failed to load")

    writer = CapturingWriter()
    monkeypatch.setattr(worker, "prepare_session", fail)
    monkeypatch.setattr(worker, "ProtocolWriter", lambda: writer)

    async def scenario() -> None:
        server = worker.Worker()
        await server._handle(initialization(tmp_path))
        assert writer.results == []
        assert [error.stable_code for error in writer.errors] == ["runtime_mismatch"]
        with pytest.raises(RpcError):
            await server._dispatch(Request("health", "runtime.health", {"sessionId": "session"}))

    asyncio.run(scenario())


@pytest.mark.parametrize("threads", [0, 5, 255])
def test_initialization_rejects_thread_budgets_outside_the_runtime_policy(tmp_path: Path, threads: int) -> None:
    params = validate_params(RuntimeInitializeParams, initialization(tmp_path, threads).params)
    with pytest.raises(RpcError) as captured:
        session.prepare_session(params)
    assert captured.value.stable_code == "unsupported_device"
