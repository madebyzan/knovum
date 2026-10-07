from __future__ import annotations

import asyncio
import io
import json
import socket
from contextlib import AsyncExitStack
from pathlib import Path
from typing import Any

import pytest
from pydantic import BaseModel

from knovum_agent_runtime.contracts.generated.control import AgentInitialize
from knovum_agent_runtime.contracts.values import GRAPH_VERSION, PROTOCOL_VERSION
from knovum_agent_runtime.server import Server
from knovum_agent_runtime.transport import HostRejected, Peer, ProtocolClosed


@pytest.mark.parametrize("obsolete", ["protocol", "previous_protocol", "graph", "previous_graph"])
def test_obsolete_runtime_is_rejected_before_opening_checkpoint(tmp_path: Path, obsolete: str) -> None:
    async def run() -> None:
        checkpoint = tmp_path / "checkpoint.sqlite"
        async with AsyncExitStack() as lifetime:
            server = Server(Peer(io.BytesIO(), io.BytesIO()), lifetime)
            request = AgentInitialize(
                protocol_version={"protocol": 0, "previous_protocol": 3}.get(obsolete, PROTOCOL_VERSION),
                graph_version={
                    "graph": "obsolete",
                    "previous_graph": GRAPH_VERSION.replace("knovum-agent-v1/", "knovum-agent-v3/"),
                }.get(obsolete, GRAPH_VERSION),
                checkpoint_path=str(checkpoint),
            )
            with pytest.raises(HostRejected, match="version-unsupported"):
                await server.initialize(request)
            assert not checkpoint.exists()

    asyncio.run(run())


class Parameters(BaseModel):
    number: int


def test_duplex_request_can_call_host_while_control_is_received() -> None:
    async def run() -> None:
        host_socket, worker_socket = socket.socketpair()
        with (
            host_socket,
            worker_socket,
            host_socket.makefile("rwb", buffering=0) as host_io,
            worker_socket.makefile("rwb", buffering=0) as worker_io,
        ):
            host, worker = Peer(host_io, host_io), Peer(worker_io, worker_io)
            control = asyncio.Event()

            async def host_handler(method: str, value: Any) -> Any:
                assert method == "admit"
                return value["number"] + 1

            async def worker_handler(method: str, value: Any) -> Any:
                if method == "control":
                    control.set()
                    return None
                admitted = await worker.request("admit", Parameters.model_validate(value))
                await control.wait()
                return admitted

            tasks = [asyncio.create_task(host.run(host_handler)), asyncio.create_task(worker.run(worker_handler))]
            try:
                execution = asyncio.create_task(host.request("execute", Parameters(number=4)))
                await host.notify("control", Parameters(number=0))
                assert await asyncio.wait_for(execution, 2) == 5
            finally:
                host.close()
                worker.close()
                host_socket.shutdown(socket.SHUT_RDWR)
                await asyncio.gather(*tasks)

    asyncio.run(run())


def test_unexpected_exception_does_not_disclose_credentials() -> None:
    async def run() -> None:
        source = io.BytesIO(b'{"jsonrpc":"2.0","id":"request","method":"fail","params":{}}\n')
        sink = io.BytesIO()
        peer = Peer(source, sink)

        async def dispatch(method: str, value: Any) -> Any:
            raise ValueError("api-key-secret")

        await peer.run(dispatch)
        response = sink.getvalue()
        assert b"api-key-secret" not in response
        assert json.loads(response)["error"]["message"] == "agent-runtime-request-failed"

    asyncio.run(run())


@pytest.mark.parametrize("frame", [b"[]\n", b'{"jsonrpc":"1.0"}\n', b'{"jsonrpc":"2.0","id":"unknown","result":NaN}\n'])
def test_invalid_external_frames_fail_visibly(frame: bytes) -> None:
    async def run() -> None:
        peer = Peer(io.BytesIO(frame), io.BytesIO())
        with pytest.raises(ValueError):
            await peer.run(lambda method, value: asyncio.sleep(0))

    asyncio.run(run())


def test_disconnected_peer_rejects_new_work() -> None:
    async def run() -> None:
        peer = Peer(io.BytesIO(), io.BytesIO())
        peer.close()
        with pytest.raises(ProtocolClosed):
            await peer.request("execute", Parameters(number=1))

    asyncio.run(run())
