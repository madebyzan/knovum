from __future__ import annotations

import asyncio
import json
from pathlib import Path
from typing import Any

import httpx
import pytest
from langchain_openai import ChatOpenAI
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver
from pydantic import ValidationError

from knovum_agent_runtime.budget import ModelBudget
from knovum_agent_runtime.contracts.generated.control import AgentExecute
from knovum_agent_runtime.execution import Execution
from knovum_agent_runtime.transport import wire_value


def request_value(workspace: Path) -> dict[str, Any]:
    return {
        "identity": {"runId": "run-1", "threadId": "thread-1", "executionRevision": "revision-1", "controlEpoch": 0},
        "model": {
            "protocol": "open_ai_chat",
            "model": "knovum-production-test",
            "baseUrl": "https://test.example/v1",
            "apiVersion": None,
            "apiKey": "test-only",
            "temperature": 0.0,
            "maxOutputTokens": 100,
            "maxInputTokens": 32768,
        },
        "systemPrompt": "Answer the user.",
        "tools": [],
        "messages": [{"id": "input:1", "role": "user", "content": [{"type": "text", "text": "hello"}]}],
        "inputId": "input:1",
        "afterCheckpointId": None,
        "resume": None,
        "skills": [],
        "skillToolBindings": {},
        "workspace": str(workspace),
    }


@pytest.mark.parametrize("field", ["plan", "research"])
def test_runtime_rejects_obsolete_composition_switches(field: str) -> None:
    value = request_value(Path("workspace"))
    value[field] = True
    with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
        AgentExecute.model_validate_json(json.dumps(value))


class Host:
    def __init__(self) -> None:
        self.calls: list[tuple[str, Any]] = []
        self.notifications: list[tuple[str, Any]] = []

    async def request(self, method: str, params: Any) -> Any:
        self.calls.append((method, wire_value(params)))
        return None

    async def notify(self, method: str, params: Any) -> None:
        self.notifications.append((method, wire_value(params)))


def test_production_composition_streams_commits_and_does_not_replay_accepted_input(tmp_path: Path, monkeypatch) -> None:
    async def run() -> None:
        requests: list[Any] = []

        def endpoint(request: httpx.Request) -> httpx.Response:
            requests.append(json.loads(request.content))
            chunks = [
                {"choices": [{"index": 0, "delta": {"role": "assistant", "content": "answer"}, "finish_reason": None}]},
                {
                    "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}],
                    "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15},
                },
            ]
            data = "".join(
                "data: "
                + json.dumps(
                    {
                        "id": "test-answer",
                        "object": "chat.completion.chunk",
                        "created": 0,
                        "model": "knovum-production-test",
                        **chunk,
                    }
                )
                + "\n\n"
                for chunk in chunks
            )
            return httpx.Response(200, headers={"content-type": "text/event-stream"}, content=data + "data: [DONE]\n\n")

        async with httpx.AsyncClient(transport=httpx.MockTransport(endpoint)) as client:

            def model(binding, purpose):
                return ChatOpenAI(
                    model=binding.model,
                    api_key=binding.api_key,
                    base_url=binding.base_url,
                    use_responses_api=False,
                    max_retries=0,
                    stream_usage=True,
                    callbacks=[ModelBudget(purpose)],
                    tags=[f"knovum:model:{purpose.value}"],
                    http_async_client=client,
                )

            monkeypatch.setattr("knovum_agent_runtime.composition.create_model", model)
            host = Host()
            async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoints.db")) as saver:
                execute = AgentExecute.model_validate_json(json.dumps(request_value(tmp_path)))
                observation = await Execution(execute, saver, host).run()
                assert observation.state.value == "completed"
                assert [value["text"] for _, value in host.notifications] == ["answer"]
                projections = [value for method, value in host.calls if method == "agentRuntime.commitProjection"]
                assert any(value["acceptedInputIds"] == ["input:1"] for value in projections)
                projected = [message for value in projections for message in value["messages"]]
                assert [message["text"] for message in projected] == ["answer"]
                assert len(requests) == 1
                assert "write_todos" in {tool["function"]["name"] for tool in requests[0]["tools"]}
                replay = request_value(tmp_path)
                replay["afterCheckpointId"] = observation.checkpoint_id
                await Execution(AgentExecute.model_validate_json(json.dumps(replay)), saver, host).run()
                assert len(requests) == 1
                admission = [value for method, value in host.calls if method == "agentRuntime.admitModel"]
                completion = [value for method, value in host.calls if method == "agentRuntime.completeModel"]
                assert len(admission) == len(completion) == 1
                assert admission[0]["checkpointId"]
                assert completion[0]["inputTokens"] == 10
                assert completion[0]["messageId"] == projected[0]["messageId"]

    asyncio.run(run())
