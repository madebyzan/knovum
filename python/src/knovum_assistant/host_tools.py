from __future__ import annotations

import json
from typing import Any

from langchain.tools import ToolRuntime
from langchain_core.messages import AIMessage, BaseMessage, HumanMessage, ToolMessage
from langchain_core.tools import StructuredTool
from langgraph.types import Command, interrupt

from knovum_assistant.contracts.generated import AssistantContentPart, AssistantToolOutcome
from knovum_assistant.contracts.generated.tools import AssistantToolDefinition, AssistantToolInvocation
from knovum_assistant.control import ExecutionContext


def content_blocks(parts: list[AssistantContentPart]) -> list[str | dict[str, Any]]:
    blocks: list[str | dict[str, Any]] = []
    for part in parts:
        value = part.root
        if value.type == "text":
            blocks.append({"type": "text", "text": value.text})
        else:
            blocks.append({"type": "image", "base64": value.data, "mime_type": value.mime_type})
    return blocks


def create_tool(definition: AssistantToolDefinition) -> StructuredTool:
    async def invoke(runtime: ToolRuntime[ExecutionContext, dict[str, Any]], **arguments: Any) -> Command[Any]:
        context = runtime.context
        message = next(message for message in reversed(runtime.state["messages"]) if isinstance(message, AIMessage))
        checkpoint_id = await context.checkpoint_id()
        if checkpoint_id is None or message.id is None or runtime.tool_call_id is None:
            raise RuntimeError("assistant-tool-intent-not-durable")
        invocation = AssistantToolInvocation(
            identity=context.identity,
            graph_namespace=runtime.config["configurable"].get("checkpoint_ns", ""),
            checkpoint_id=checkpoint_id,
            message_id=message.id,
            tool_call_id=runtime.tool_call_id,
            name=definition.name,
            arguments=arguments,
        )
        response = await context.peer.request("assistant.invokeTool", invocation)
        outcome = AssistantToolOutcome.model_validate_json(json.dumps(response)).root
        if outcome.type == "await_child":
            interrupt({"type": "child_run", "runId": outcome.run_id})
            # Resume must re-observe the original Action receipt through Rust.
            response = await context.peer.request("assistant.invokeTool", invocation)
            outcome = AssistantToolOutcome.model_validate_json(json.dumps(response)).root
            if outcome.type != "completed":
                raise RuntimeError("assistant-child-not-completed")
        tool_message = ToolMessage(
            id=f"tool:{runtime.tool_call_id}",
            content=json.dumps(outcome.value, ensure_ascii=False),
            tool_call_id=runtime.tool_call_id,
            name=definition.name,
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
