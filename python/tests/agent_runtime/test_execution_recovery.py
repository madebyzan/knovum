from __future__ import annotations

import asyncio
import json
from pathlib import Path

import httpx
import pytest
from agent_runtime.production_fixtures import LedgerHost, bind_script
from agent_runtime.sdk_fixtures import answer
from agent_runtime.test_execution import request_value
from langchain_openai import ChatOpenAI
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver
from openai import InternalServerError

from knovum_agent_runtime.budget import ModelBudget
from knovum_agent_runtime.contracts.generated.control import AgentExecute
from knovum_agent_runtime.execution import Execution
from knovum_agent_runtime.transport import HostRejected


@pytest.mark.parametrize("summary", [False, True])
def test_unknown_provider_failure_prevents_hidden_sdk_retries(tmp_path: Path, monkeypatch, summary: bool) -> None:
    async def run() -> None:
        requests: list[httpx.Request] = []

        def endpoint(request: httpx.Request) -> httpx.Response:
            requests.append(request)
            return httpx.Response(500, json={"error": {"message": "provider failed", "type": "server_error"}})

        async with httpx.AsyncClient(transport=httpx.MockTransport(endpoint)) as client:

            def model(binding, purpose):
                return ChatOpenAI(
                    model=binding.model,
                    api_key=binding.api_key,
                    base_url=binding.base_url,
                    use_responses_api=False,
                    max_retries=0,
                    callbacks=[ModelBudget(purpose, binding.max_output_tokens)],
                    tags=[f"knovum:model:{purpose.value}"],
                    http_async_client=client,
                )

            monkeypatch.setattr("knovum_agent_runtime.composition.create_model", model)
            value = request_value()
            if summary:
                value["model"]["maxInputTokens"] = 8192
                value["messages"].insert(
                    0,
                    {"id": "history:large", "role": "user", "content": [{"type": "text", "text": "older " * 20000}]},
                )
            host = LedgerHost()
            async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
                execution = Execution(AgentExecute.model_validate_json(json.dumps(value)), saver, host)
                with pytest.raises((HostRejected, InternalServerError)):
                    await execution.run()
                assert len(requests) == 1
                assert set(host.states.values()) == {"unknown"}
                admissions = [params for method, params in host.calls if method == "agentRuntime.admitModel"]
                assert admissions[0]["purpose"] == ("context_summary" if summary else "main")
                with pytest.raises(HostRejected, match="model-outcome-unknown"):
                    await Execution(AgentExecute.model_validate_json(json.dumps(value)), saver, host).run()
                assert len(requests) == 1

    asyncio.run(run())


def test_lost_projection_ack_recovers_accepted_input_without_another_model_call(tmp_path: Path, monkeypatch) -> None:
    class LostAckHost(LedgerHost):
        lost = False

        async def request(self, method, params):
            result = await super().request(method, params)
            if method == "agentRuntime.commitProjection" and self.calls[-1][1]["messages"] and not self.lost:
                self.lost = True
                raise HostRejected(-32000, "projection-ack-lost")
            return result

    async def run() -> None:
        models = bind_script(
            monkeypatch,
            {
                "main": [answer("durable answer", message_id="answer")],
                "context_summary": [answer("unused", message_id="unused")],
            },
        )
        host = LostAckHost()
        value = request_value()
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            with pytest.raises(HostRejected, match="projection-ack-lost"):
                await Execution(AgentExecute.model_validate_json(json.dumps(value)), saver, host).run()
            first_model = models["main"]
            result = await Execution(AgentExecute.model_validate_json(json.dumps(value)), saver, host).run()
            assert result.state.value == "completed"
            assert len(first_model.requests) == 1
            assert not models["main"].requests
            assert set(host.states.values()) == {"committed"}

    asyncio.run(run())
