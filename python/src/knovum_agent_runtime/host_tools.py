from __future__ import annotations

import json
from typing import Any

from langchain.tools import ToolRuntime
from langchain_core.messages import AIMessage, BaseMessage, HumanMessage, ToolMessage
from langchain_core.tools import StructuredTool
from langgraph.types import Command, interrupt

from knovum_agent_runtime.contracts.generated import AgentContentPart, AgentToolOutcome
from knovum_agent_runtime.contracts.generated.tools import AgentToolDefinition, AgentToolInvocation
from knovum_agent_runtime.control import ExecutionContext


def content_blocks(parts: list[AgentContentPart]) -> list[str | dict[str, Any]]:
    blocks: list[str | dict[str, Any]] = []
    for part in parts:
        value = part.root
        if value.type == "text":
            blocks.append({"type": "text", "text": value.text})
        else:
            blocks.append({"type": "image", "base64": value.data, "mime_type": value.mime_type})
    return blocks


def create_tool(definition: AgentToolDefinition) -> StructuredTool:
    async def invoke(runtime: ToolRuntime[ExecutionContext, dict[str, Any]], **arguments: Any) -> Command[Any]:
        context = runtime.context
        message = next(message for message in reversed(runtime.state["messages"]) if isinstance(message, AIMessage))
        checkpoint_id = await context.checkpoint_id()
        if checkpoint_id is None or message.id is None or runtime.tool_call_id is None:
            raise RuntimeError("agent-runtime-tool-intent-not-durable")
        invocation = AgentToolInvocation(
            identity=context.identity,
            graph_namespace=runtime.config["configurable"].get("checkpoint_ns", ""),
            checkpoint_id=checkpoint_id,
            message_id=message.id,
            tool_call_id=runtime.tool_call_id,
            name=definition.name,
            arguments=arguments,
        )
        while True:
            response = await context.peer.request("agentRuntime.invokeTool", invocation)
            outcome = AgentToolOutcome.model_validate(response).root
            if outcome.type == "completed":
                break
            if outcome.type == "await_child":
                interrupt({"type": "child_run", "runId": outcome.run_id})
            else:
                interrupt({"type": "tool_intervention", "operationId": outcome.operation_id, "reason": outcome.reason})
        tool_message = ToolMessage(
            id=f"tool:{runtime.tool_call_id}",
            content=json.dumps(outcome.value, ensure_ascii=False),
            tool_call_id=runtime.tool_call_id,
            name=definition.name,
            status="error" if isinstance(outcome.value, dict) and outcome.value.get("isError") is True else "success",
        )
        messages: list[BaseMessage] = [tool_message]
        if outcome.media:
            messages.append(HumanMessage(id=f"media:{runtime.tool_call_id}", content=content_blocks(outcome.media)))
        return Command(update={"messages": messages})

    return StructuredTool.from_function(
        coroutine=invoke,
        name=definition.name,
        description=definition.description,
        args_schema=definition.parameters,
    )
