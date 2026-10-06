from __future__ import annotations

from collections.abc import AsyncIterator
from typing import Any

from deepagents.profiles import GeneralPurposeSubagentProfile, HarnessProfile, register_harness_profile
from langchain_core.callbacks import AsyncCallbackHandler
from langchain_core.messages import AIMessage, AIMessageChunk, BaseMessage
from langchain_core.outputs import ChatGeneration, ChatGenerationChunk, ChatResult
from langchain_openai import ChatOpenAI
from langgraph.runtime import get_runtime
from pydantic import Field, PrivateAttr


class AdmissionRecorder(AsyncCallbackHandler):
    raise_error = True
    run_inline = True

    def __init__(self, *, deny: bool = False) -> None:
        self.deny = deny
        self.starts: list[str] = []
        self.completions: list[str] = []
        self.contexts: list[Any] = []
        self.checkpoints: list[str | None] = []
        self.checkpoint_reader: Any = None

    async def on_chat_model_start(
        self, serialized: dict[str, Any], messages: list[list[BaseMessage]], **kwargs: Any
    ) -> None:
        self.starts.append(str(kwargs["run_id"]))
        self.contexts.append(get_runtime().context)
        if self.checkpoint_reader is not None:
            snapshot = await self.checkpoint_reader()
            self.checkpoints.append(snapshot.config["configurable"]["checkpoint_id"])
        if self.deny:
            raise PermissionError("model admission denied")

    async def on_llm_end(self, response: Any, **kwargs: Any) -> None:
        self.completions.append(str(kwargs["run_id"]))


class ScriptedModel(ChatOpenAI):
    """Exercise the actual SDK binding/callback lifecycle without a provider request."""

    script: list[AIMessage] = Field(exclude=True)
    _requests: list[list[BaseMessage]] = PrivateAttr(default_factory=list)
    _tools: list[set[str]] = PrivateAttr(default_factory=list)

    @property
    def requests(self) -> list[list[BaseMessage]]:
        return self._requests

    @property
    def advertised_tools(self) -> list[set[str]]:
        return self._tools

    async def _agenerate(self, messages: list[BaseMessage], stop: list[str] | None = None, **kwargs: Any) -> ChatResult:
        self._requests.append(messages)
        self._tools.append({value["function"]["name"] for value in kwargs.get("tools", [])})
        index = min(len(self._requests) - 1, len(self.script) - 1)
        message = self.script[index].model_copy(deep=True)
        return ChatResult(generations=[ChatGeneration(message=message)])

    async def _astream(
        self, messages: list[BaseMessage], stop: list[str] | None = None, **kwargs: Any
    ) -> AsyncIterator[ChatGenerationChunk]:
        result = await self._agenerate(messages, stop=stop, **kwargs)
        message = result.generations[0].message
        yield ChatGenerationChunk(message=AIMessageChunk(**message.model_dump(exclude={"type"})))


def scripted_model(script: list[AIMessage], recorder: AdmissionRecorder | None = None) -> ScriptedModel:
    register_harness_profile(
        "openai:knovum-a0",
        HarnessProfile(general_purpose_subagent=GeneralPurposeSubagentProfile(enabled=False)),
    )
    return ScriptedModel(
        model="knovum-a0",
        api_key="a0-test-credential",
        use_responses_api=False,
        max_retries=0,
        script=script,
        callbacks=[recorder] if recorder else [],
    )


def answer(text: str, *, message_id: str) -> AIMessage:
    return AIMessage(
        content=text,
        id=message_id,
        usage_metadata={"input_tokens": 10, "output_tokens": 5, "total_tokens": 15},
    )
