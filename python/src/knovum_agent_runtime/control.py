from __future__ import annotations

import asyncio
from collections.abc import Awaitable, Callable
from dataclasses import dataclass, field
from operator import add
from typing import Annotated, Any, cast

from langchain.agents.middleware import (
    AgentMiddleware,
    AgentState,
    ExtendedModelResponse,
    ModelRequest,
    ModelResponse,
    ToolCallRequest,
)
from langchain_core.messages import AIMessage, ToolMessage
from langchain_core.tools import BaseTool
from langgraph.config import get_config
from langgraph.graph.state import CompiledStateGraph
from langgraph.runtime import Runtime
from langgraph.types import Command, interrupt

from knovum_agent_runtime.contracts.generated.control import AgentExecutionIdentity
from knovum_agent_runtime.contracts.generated.tools import AgentToolAdmission
from knovum_agent_runtime.projection import ProjectionPublisher
from knovum_agent_runtime.transport import Peer


@dataclass
class ExecutionContext:
    identity: AgentExecutionIdentity
    peer: Peer
    graph: CompiledStateGraph[Any, Any, Any, Any]
    projection: ProjectionPublisher
    paused: bool = False
    cancelled: bool = False
    admitted_requests: dict[str, str] = field(default_factory=dict)
    completed_requests: dict[str, list[str]] = field(default_factory=dict)
    research_slots: asyncio.Semaphore = field(default_factory=lambda: asyncio.Semaphore(2))

    def boundary(self) -> None:
        if self.cancelled:
            raise asyncio.CancelledError("agent-runtime-cancelled")
        if self.paused:
            interrupt({"type": "pause", "runId": self.identity.run_id})

    async def checkpoint_id(self) -> str | None:
        state = await self.graph.aget_state({"configurable": {"thread_id": self.identity.thread_id}})
        return state.config["configurable"].get("checkpoint_id")


class CheckpointState(AgentState):
    model_receipts: Annotated[list[str], add]


class ControlMiddleware(AgentMiddleware[CheckpointState, ExecutionContext]):
    state_schema = CheckpointState

    def __init__(self, *, read_only: bool = False) -> None:
        self.read_only = read_only

    async def awrap_model_call(
        self,
        request: ModelRequest[ExecutionContext],
        handler: Callable[[ModelRequest[ExecutionContext]], Awaitable[ModelResponse[Any]]],
    ) -> ExtendedModelResponse[Any]:
        if self.read_only:
            excluded = {"write_file", "edit_file", "delete", "execute", "task", "write_todos"}
            tools = [
                tool
                for tool in request.tools
                if (tool.name if isinstance(tool, BaseTool) else tool["name"]) not in excluded
            ]
            request = request.override(tools=tools)
        response = await handler(request)
        # Persist the result and its admission receipts in the same model-node checkpoint.
        return ExtendedModelResponse(response, Command(update=self.receipts(request.runtime.context)))

    async def abefore_model(self, state: CheckpointState, runtime: Runtime[ExecutionContext]) -> dict[str, Any]:
        runtime.context.boundary()
        return self.receipts(runtime.context)

    @staticmethod
    def receipts(context: ExecutionContext) -> dict[str, Any]:
        namespace = get_config()["configurable"].get("checkpoint_ns", "").rpartition("|")[0]
        return {"model_receipts": context.completed_requests.pop(namespace, [])}

    async def awrap_tool_call(
        self,
        request: ToolCallRequest,
        handler: Callable[[ToolCallRequest], Awaitable[ToolMessage | Command[Any]]],
    ) -> ToolMessage | Command[Any]:
        context = cast(ExecutionContext, request.runtime.context)
        context.boundary()
        message = next(message for message in reversed(request.state["messages"]) if isinstance(message, AIMessage))
        if message.id is None or request.tool_call["id"] is None:
            raise RuntimeError("agent-runtime-tool-intent-not-durable")
        await context.peer.request(
            "agentRuntime.admitTool",
            AgentToolAdmission(
                sibling_tools=[call["name"] for call in message.tool_calls],
                identity=context.identity,
                graph_namespace=request.runtime.config["configurable"].get("checkpoint_ns", ""),
                message_id=message.id,
                tool_call_id=request.tool_call["id"],
                name=request.tool_call["name"],
            ),
        )
        if request.tool_call["name"] == "task":
            async with context.research_slots:
                context.boundary()
                return await handler(request)
        return await handler(request)
