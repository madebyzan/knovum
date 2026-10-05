from __future__ import annotations

import asyncio
import threading
import time
from pathlib import Path

import pytest

from knovum_inference.contracts.generated.control import RuntimeCapability
from knovum_inference.errors import RpcError
from knovum_inference.runtime.session import Session
from knovum_inference.runtime.worker import Worker
from knovum_inference.transport.protocol import Request


def embedding_request() -> Request:
    return Request(
        "request",
        "embedding.encode",
        {
            "requestId": "request",
            "attemptEpoch": 1,
            "ownerKind": "test",
            "ownerId": "owner",
            "attemptId": "attempt",
            "inputRevision": "revision",
            "modelSpecId": "embedding-spec",
            "deadlineMs": int(time.time() * 1000) + 10_000,
            "embeddingSpecId": "embedding-spec",
            "texts": ["text"],
            "role": "query",
        },
    )


@pytest.mark.parametrize("outcome", ["success", "failure", "invalid_output", "cancelled", "deadline_exceeded"])
def test_inference_always_settles_its_active_request(
    outcome: str, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    async def scenario() -> None:
        started = threading.Event()
        release = threading.Event()
        server = Worker()

        class Engine:
            def encode(self, texts: list[str], role: str) -> tuple[list[list[float]], int]:
                started.set()
                assert release.wait(3)
                if outcome == "failure":
                    raise MemoryError("native allocation failed")
                return [[float("nan") if outcome == "invalid_output" else 0.25]], 1

        server._session = Session("session", RuntimeCapability.embedding, "embedding-spec", Engine(), tmp_path)
        request = embedding_request()
        task = asyncio.create_task(server._dispatch(request))
        try:
            assert await asyncio.to_thread(started.wait, 2)
            active = server._active
            assert active is not None
            if outcome == "cancelled":
                await server._dispatch(
                    Request(
                        "cancel",
                        "runtime.cancel",
                        {
                            "targetRequestId": "request",
                            "attemptEpoch": 1,
                        },
                    )
                )
            elif outcome == "deadline_exceeded":
                elapsed_seconds = (request.params["deadlineMs"] + 1_000) / 1_000
                monkeypatch.setattr(time, "time", lambda: elapsed_seconds)
            release.set()
            if outcome == "success":
                result = await task
                assert result["vectors"] == [[0.25]]
                assert result["requestId"] == "request"
            elif outcome == "failure":
                with pytest.raises(MemoryError):
                    await task
            else:
                with pytest.raises(RpcError) as captured:
                    await task
                assert captured.value.stable_code == outcome
            assert server._active is None
            assert active.done.is_set()
        finally:
            release.set()
            if not task.done():
                await task

    asyncio.run(scenario())


def test_invalid_embedding_input_never_reserves_the_worker(tmp_path: Path) -> None:
    async def scenario() -> None:
        server = Worker()
        server._session = Session("session", RuntimeCapability.embedding, "embedding-spec", object(), tmp_path)
        request = embedding_request()
        request.params["texts"] = [""]
        with pytest.raises(RpcError) as captured:
            await server._dispatch(request)
        assert captured.value.code == -32602
        assert captured.value.stable_code == "invalid_output"
        assert server._active is None

    asyncio.run(scenario())
