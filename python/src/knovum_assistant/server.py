from __future__ import annotations

import asyncio
import json
from contextlib import AsyncExitStack
from datetime import datetime
from pathlib import Path
from typing import Any, TypeVar

from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver
from pydantic import BaseModel, ValidationError

from knovum_assistant.contracts.generated.control import (
    AssistantControl,
    AssistantControlOperation,
    AssistantExecute,
    AssistantInitialize,
    AssistantPruneThreads,
    AssistantReady,
)
from knovum_assistant.contracts.generated.messages import AssistantProjectionRequest
from knovum_assistant.contracts.values import GRAPH_VERSION, PROTOCOL_VERSION
from knovum_assistant.execution import Execution
from knovum_assistant.projection import export_projection
from knovum_assistant.transport import HostRejected, Peer

Model = TypeVar("Model", bound=BaseModel)


def parameters(model: type[Model], value: Any) -> Model:
    try:
        return model.model_validate_json(json.dumps(value, allow_nan=False))
    except (ValidationError, ValueError):
        raise HostRejected(-32602, "assistant-invalid-parameters") from None


class Server:
    def __init__(self, peer: Peer, lifetime: AsyncExitStack) -> None:
        self.peer = peer
        self.lifetime = lifetime
        self.saver: AsyncSqliteSaver | None = None
        self.execution: Execution | None = None

    async def dispatch(self, method: str, value: Any) -> Any:
        match method:
            case "assistant.initialize":
                return await self.initialize(parameters(AssistantInitialize, value))
            case "assistant.execute":
                if self.saver is None or (self.execution is not None and self.execution.task is not None):
                    raise HostRejected(-32000, "assistant-not-ready")
                self.execution = Execution(parameters(AssistantExecute, value), self.saver, self.peer)
                return await self.execution.run()
            case "assistant.control":
                control = parameters(AssistantControl, value)
                if self.execution is None or control.identity != self.execution.request.identity:
                    raise HostRejected(-32000, "assistant-execution-identity-mismatch")
                if control.operation == AssistantControlOperation.cancel:
                    self.execution.cancel()
                else:
                    self.execution.context.paused = True
                return None
            case "assistant.projection":
                projection = parameters(AssistantProjectionRequest, value)
                if self.execution is None or projection.thread_id != self.execution.request.identity.thread_id:
                    raise HostRejected(-32000, "assistant-thread-unbound")
                return await export_projection(
                    self.execution.graph, self.execution.saver, projection.thread_id, projection.after_checkpoint_id
                )
            case "assistant.pruneThreads":
                prune = parameters(AssistantPruneThreads, value)
                if self.saver is None or (self.execution is not None and self.execution.task is not None):
                    raise HostRejected(-32000, "assistant-not-idle")
                retained = set(prune.retained_thread_ids)
                latest: dict[str, float] = {}
                async for checkpoint in self.saver.alist(None):
                    thread = checkpoint.config["configurable"]["thread_id"]
                    if thread not in retained:
                        timestamp = datetime.fromisoformat(checkpoint.checkpoint["ts"]).timestamp() * 1000
                        latest[thread] = max(latest.get(thread, timestamp), timestamp)
                expired = {thread for thread, timestamp in latest.items() if timestamp < prune.orphan_before_ms}
                for thread_id in set(prune.thread_ids) | expired:
                    await self.saver.adelete_thread(thread_id)
                return None
            case "assistant.shutdown":
                if self.execution is not None:
                    self.execution.cancel()
                self.peer.close()
                return None
        raise HostRejected(-32601, "assistant-method-not-found")

    async def initialize(self, request: AssistantInitialize) -> AssistantReady:
        if self.saver is not None:
            raise HostRejected(-32000, "assistant-already-initialized")
        if request.protocol_version != PROTOCOL_VERSION or request.graph_version != GRAPH_VERSION:
            raise HostRejected(-32000, "assistant-version-unsupported")
        path = Path(request.checkpoint_path)
        if not path.is_absolute():
            raise HostRejected(-32602, "assistant-checkpoint-path-invalid")
        path.parent.mkdir(parents=True, exist_ok=True)
        self.saver = await self.lifetime.enter_async_context(AsyncSqliteSaver.from_conn_string(str(path)))
        await self.saver.setup()
        return AssistantReady(protocol_version=PROTOCOL_VERSION, graph_version=GRAPH_VERSION)


async def serve() -> None:
    async with AsyncExitStack() as lifetime:
        peer = Peer()
        server = Server(peer, lifetime)
        await peer.run(server.dispatch)


def main() -> None:
    asyncio.run(serve())
