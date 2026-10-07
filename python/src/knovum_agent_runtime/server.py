from __future__ import annotations

import asyncio
import json
from contextlib import AsyncExitStack
from pathlib import Path
from typing import Any, TypeVar

from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver
from pydantic import BaseModel, ValidationError

from knovum_agent_runtime.contracts.generated.control import (
    AgentControl,
    AgentControlOperation,
    AgentExecute,
    AgentInitialize,
    AgentPruneThreads,
    AgentReady,
)
from knovum_agent_runtime.contracts.generated.messages import AgentProjectionRequest
from knovum_agent_runtime.contracts.values import GRAPH_VERSION, PROTOCOL_VERSION
from knovum_agent_runtime.execution import Execution
from knovum_agent_runtime.projection import export_projection
from knovum_agent_runtime.retention import prune_checkpoints
from knovum_agent_runtime.transport import HostRejected, Peer

Model = TypeVar("Model", bound=BaseModel)


def parameters(model: type[Model], value: Any) -> Model:
    try:
        return model.model_validate_json(json.dumps(value, allow_nan=False))
    except (ValidationError, ValueError):
        raise HostRejected(-32602, "agent-runtime-invalid-parameters") from None


class Server:
    def __init__(self, peer: Peer, lifetime: AsyncExitStack) -> None:
        self.peer = peer
        self.lifetime = lifetime
        self.saver: AsyncSqliteSaver | None = None
        self.execution: Execution | None = None

    async def dispatch(self, method: str, value: Any) -> Any:
        match method:
            case "agentRuntime.initialize":
                return await self.initialize(parameters(AgentInitialize, value))
            case "agentRuntime.execute":
                if self.saver is None or (self.execution is not None and self.execution.task is not None):
                    raise HostRejected(-32000, "agent-runtime-not-ready")
                self.execution = Execution(parameters(AgentExecute, value), self.saver, self.peer)
                return await self.execution.run()
            case "agentRuntime.control":
                control = parameters(AgentControl, value)
                if self.execution is None or control.identity != self.execution.request.identity:
                    raise HostRejected(-32000, "agent-runtime-execution-identity-mismatch")
                if control.operation == AgentControlOperation.cancel:
                    self.execution.cancel()
                else:
                    self.execution.context.paused = True
                return None
            case "agentRuntime.projection":
                projection = parameters(AgentProjectionRequest, value)
                if self.execution is None or projection.thread_id != self.execution.request.identity.thread_id:
                    raise HostRejected(-32000, "agent-runtime-thread-unbound")
                return await export_projection(
                    self.execution.graph, self.execution.saver, projection.thread_id, projection.after_checkpoint_id
                )
            case "agentRuntime.pruneThreads":
                prune = parameters(AgentPruneThreads, value)
                if self.saver is None or (self.execution is not None and self.execution.task is not None):
                    raise HostRejected(-32000, "agent-runtime-not-idle")
                await prune_checkpoints(self.saver, prune)
                return None
            case "agentRuntime.shutdown":
                if self.execution is not None:
                    self.execution.cancel()
                self.peer.close()
                return None
        raise HostRejected(-32601, "agent-runtime-method-not-found")

    async def initialize(self, request: AgentInitialize) -> AgentReady:
        if self.saver is not None:
            raise HostRejected(-32000, "agent-runtime-already-initialized")
        if request.protocol_version != PROTOCOL_VERSION or request.graph_version != GRAPH_VERSION:
            raise HostRejected(-32000, "agent-runtime-version-unsupported")
        path = Path(request.checkpoint_path)
        if not path.is_absolute():
            raise HostRejected(-32602, "agent-runtime-checkpoint-path-invalid")
        path.parent.mkdir(parents=True, exist_ok=True)
        self.saver = await self.lifetime.enter_async_context(AsyncSqliteSaver.from_conn_string(str(path)))
        await self.saver.setup()
        return AgentReady(protocol_version=PROTOCOL_VERSION, graph_version=GRAPH_VERSION)


async def serve() -> None:
    async with AsyncExitStack() as lifetime:
        peer = Peer()
        server = Server(peer, lifetime)
        await peer.run(server.dispatch)


def main() -> None:
    asyncio.run(serve())
