from __future__ import annotations

from typing import Any

from langchain_core.messages import AIMessage, HumanMessage
from langchain_core.runnables import RunnableConfig
from langgraph.graph.state import CompiledStateGraph
from langgraph.types import Command

from knovum_assistant.contracts.generated.control import AssistantExecute
from knovum_assistant.host_tools import content_blocks


async def execution_input(request: AssistantExecute, graph: CompiledStateGraph[Any, Any, Any, Any]) -> Any:
    if request.resume is not None:
        if request.messages:
            raise ValueError("assistant-resume-cannot-append-input")
        return Command(resume=request.resume)
    if not request.messages:
        return None
    if (
        not request.input_id.startswith("input:")
        or sum(message.id == request.input_id for message in request.messages) != 1
    ):
        raise ValueError("assistant-input-identity-invalid")
    config: RunnableConfig = {"configurable": {"thread_id": request.identity.thread_id}}
    async for state in graph.aget_state_history(config):
        if any(message.id == request.input_id for message in state.values.get("messages", [])):
            # The host may resend an input whose checkpoint ACK was lost. Continue
            # pending nodes; never append or re-execute an already completed turn.
            return None
    return {
        "messages": [
            HumanMessage(id=message.id, content=content_blocks(message.content))
            if message.role.value == "user"
            else AIMessage(id=message.id, content=content_blocks(message.content))
            for message in request.messages
        ]
    }
