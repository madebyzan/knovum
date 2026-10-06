from __future__ import annotations

import asyncio
from collections.abc import Sequence
from typing import Any

from langchain_core.messages import AIMessage, BaseMessage, HumanMessage
from langchain_core.runnables import RunnableConfig
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver
from langgraph.graph.state import CompiledStateGraph

from knovum_agent_runtime.checkpoints import latest_namespace_checkpoints
from knovum_agent_runtime.contracts.generated.messages import (
    AgentPlanItem,
    AgentPlanState,
    AgentProjectedMessage,
    AgentProjection,
)
from knovum_agent_runtime.contracts.generated.models import AgentModelCheckpoint
from knovum_agent_runtime.transport import Peer


def visible_text(message: BaseMessage) -> str:
    if isinstance(message.content, str):
        return message.content
    return "".join(
        block["text"] for block in message.content if isinstance(block, dict) and block.get("type") == "text"
    )


async def export_projection(
    graph: CompiledStateGraph[Any, Any, Any, Any],
    saver: AsyncSqliteSaver,
    thread_id: str,
    after_checkpoint_id: str | None,
) -> AgentProjection:
    config: RunnableConfig = {"configurable": {"thread_id": thread_id}}
    latest = await graph.aget_state(config)
    snapshots = []
    latest_checkpoint_id: str | None = None
    previous_message_ids: set[str] = set()
    found_cursor = after_checkpoint_id is None
    async for state in graph.aget_state_history(config):
        checkpoint_id = state.config["configurable"]["checkpoint_id"]
        if latest_checkpoint_id is None:
            latest_checkpoint_id = checkpoint_id
        if checkpoint_id == after_checkpoint_id:
            found_cursor = True
            previous_message_ids = {
                message.id
                for message in state.values.get("messages", [])
                if isinstance(message, AIMessage) and message.id
            }
            break
        snapshots.append(state)
    if latest_checkpoint_id is None:
        raise ValueError("agent-runtime-thread-not-found")
    if not found_cursor:
        raise ValueError("agent-runtime-projection-cursor-not-found")
    messages: dict[str, AgentProjectedMessage] = {}
    input_ids: dict[str, None] = {}
    for state in reversed(snapshots):
        append_messages(state.values.get("messages", []), messages, input_ids)
    model_checkpoints = []
    async for checkpoint in latest_namespace_checkpoints(saver, thread_id):
        namespace = checkpoint.config["configurable"]["checkpoint_ns"]
        values = checkpoint.checkpoint["channel_values"]
        receipts = list(values.get("model_receipts", []))
        model_messages = list(values.get("messages", []))
        for _, channel, value in checkpoint.pending_writes or []:
            if channel == "model_receipts":
                receipts.extend(value)
            elif channel == "messages":
                model_messages.extend(value if isinstance(value, list) else [value])
        model_checkpoints.append(
            AgentModelCheckpoint(
                graph_namespace=namespace,
                message_ids=[message.id for message in model_messages if isinstance(message, AIMessage) and message.id],
                request_ids=list(dict.fromkeys(receipts)),
            )
        )
    return AgentProjection(
        thread_id=thread_id,
        checkpoint_id=latest_checkpoint_id,
        messages=[message for message_id, message in messages.items() if message_id not in previous_message_ids],
        accepted_input_ids=list(input_ids),
        model_checkpoints=model_checkpoints,
        plan=[
            AgentPlanItem(content=item["content"], state=AgentPlanState(item["status"]))
            for item in latest.values.get("todos", [])
        ],
    )


def append_messages(
    source: Sequence[BaseMessage],
    messages: dict[str, AgentProjectedMessage],
    input_ids: dict[str, None],
) -> None:
    for message in source:
        if isinstance(message, HumanMessage) and message.id and message.id.startswith("input:"):
            input_ids[message.id] = None
        if (
            isinstance(message, AIMessage)
            and message.id
            and not message.id.startswith("history:")
            and visible_text(message)
        ):
            messages[message.id] = AgentProjectedMessage(message_id=message.id, text=visible_text(message))


class ProjectionPublisher:
    def __init__(
        self,
        graph: CompiledStateGraph[Any, Any, Any, Any],
        saver: AsyncSqliteSaver,
        peer: Peer,
        thread_id: str,
        cursor: str | None,
    ) -> None:
        self.graph = graph
        self.saver = saver
        self.peer = peer
        self.thread_id = thread_id
        self.cursor = cursor
        self._lock = asyncio.Lock()

    async def commit(self) -> None:
        async with self._lock:
            projection = await export_projection(self.graph, self.saver, self.thread_id, self.cursor)
            await self.peer.request("agentRuntime.commitProjection", projection)
            self.cursor = projection.checkpoint_id
