from __future__ import annotations

from typing import Any, cast
from uuid import UUID

from langchain_core.callbacks import AsyncCallbackHandler
from langchain_core.messages import AIMessage, BaseMessage
from langchain_core.outputs import ChatGeneration, LLMResult
from langgraph.runtime import get_runtime

from knovum_agent_runtime.contracts.generated.models import (
    AgentModelAdmission,
    AgentModelCompletion,
    AgentModelFailure,
    AgentModelPurpose,
)
from knovum_agent_runtime.control import ExecutionContext


class ModelBudget(AsyncCallbackHandler):
    raise_error = True
    run_inline = True

    def __init__(self, purpose: AgentModelPurpose) -> None:
        self.purpose = purpose

    async def on_chat_model_start(
        self,
        serialized: dict[str, Any],
        messages: list[list[BaseMessage]],
        *,
        run_id: UUID,
        metadata: dict[str, Any] | None = None,
        **kwargs: Any,
    ) -> None:
        context = cast(ExecutionContext, get_runtime().context)
        context.boundary()
        request_id = str(run_id)
        await context.projection.commit()
        await context.peer.request(
            "agentRuntime.admitModel",
            AgentModelAdmission(
                identity=context.identity,
                request_id=request_id,
                purpose=self.purpose,
                checkpoint_id=await context.checkpoint_id(),
                graph_namespace=(metadata or {}).get("langgraph_checkpoint_ns", ""),
            ),
        )
        context.admitted_requests[request_id] = (metadata or {}).get("langgraph_checkpoint_ns", "").rpartition("|")[0]

    async def on_llm_end(self, response: LLMResult, *, run_id: UUID, **kwargs: Any) -> None:
        context = cast(ExecutionContext, get_runtime().context)
        message = cast(AIMessage, cast(ChatGeneration, response.generations[0][0]).message)
        usage = message.usage_metadata
        request_id = str(run_id)
        await context.peer.request(
            "agentRuntime.completeModel",
            AgentModelCompletion(
                identity=context.identity,
                request_id=request_id,
                input_tokens=usage["input_tokens"] if usage is not None else None,
                output_tokens=usage["output_tokens"] if usage is not None else None,
                message_id=message.id,
            ),
        )
        namespace = context.admitted_requests.pop(request_id)
        context.completed_requests.setdefault(namespace, []).append(request_id)

    async def on_llm_error(self, error: BaseException, *, run_id: UUID, **kwargs: Any) -> None:
        context = cast(ExecutionContext, get_runtime().context)
        request_id = str(run_id)
        if request_id in context.admitted_requests:
            await context.peer.request(
                "agentRuntime.failModel",
                AgentModelFailure(identity=context.identity, request_id=request_id),
            )
            context.admitted_requests.pop(request_id)
