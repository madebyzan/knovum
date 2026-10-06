from __future__ import annotations

import asyncio
from typing import Any, cast

from langchain_core.messages import BaseMessage
from langchain_core.runnables import RunnableConfig
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver

from knovum_assistant.composition import create_assistant
from knovum_assistant.contracts.generated.control import (
    AssistantExecute,
    AssistantExecutionResult,
    AssistantExecutionState,
)
from knovum_assistant.contracts.generated.messages import AssistantContentDelta
from knovum_assistant.control import ExecutionContext
from knovum_assistant.inputs import execution_input
from knovum_assistant.projection import ProjectionPublisher, visible_text
from knovum_assistant.transport import Peer


class Execution:
    def __init__(self, request: AssistantExecute, saver: AsyncSqliteSaver, peer: Peer) -> None:
        self.request = request
        self.peer = peer
        self.saver = saver
        self.graph = create_assistant(request, saver)
        self.publisher = ProjectionPublisher(
            self.graph, saver, peer, request.identity.thread_id, request.after_checkpoint_id
        )
        self.context = ExecutionContext(request.identity, peer, graph=self.graph, projection=self.publisher)
        self.task: asyncio.Task[Any] | None = None

    async def run(self) -> AssistantExecutionResult:
        self.task = asyncio.current_task()
        config: RunnableConfig = {
            "configurable": {"thread_id": self.request.identity.thread_id},
            "recursion_limit": 512,
        }
        try:
            previous = await self.graph.aget_state(config)
            if previous.config["configurable"].get("checkpoint_id"):
                await self.publisher.commit()
            await self.peer.request("assistant.requireRecovery", self.request.identity)
            inputs = await execution_input(self.request, self.graph)
            async for event in self.graph.astream(
                inputs,
                config,
                context=self.context,
                stream_mode=["messages", "updates"],
                subgraphs=True,
                durability="sync",
            ):
                namespace, mode, payload = cast(tuple[tuple[str, ...], str, Any], event)
                if namespace:
                    continue
                if mode == "messages":
                    message, metadata = cast(tuple[BaseMessage, dict[str, Any]], payload)
                    if "knovum:model:main" in metadata.get("tags", []):
                        text = visible_text(message)
                        if text and message.id:
                            await self.peer.notify(
                                "assistant.contentDelta",
                                AssistantContentDelta(identity=self.request.identity, message_id=message.id, text=text),
                            )
                elif mode == "updates":
                    await self.publisher.commit()
            snapshot = await self.graph.aget_state(config)
            await self.publisher.commit()
            interrupts = [item.value for task in snapshot.tasks for item in task.interrupts]
            return AssistantExecutionResult(
                identity=self.request.identity,
                checkpoint_id=snapshot.config["configurable"]["checkpoint_id"],
                state=AssistantExecutionState.interrupted if interrupts else AssistantExecutionState.completed,
                interrupts=interrupts,
            )
        finally:
            self.task = None

    def cancel(self) -> None:
        self.context.cancelled = True
        if self.task is not None:
            self.task.cancel()
