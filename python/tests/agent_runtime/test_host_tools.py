from __future__ import annotations

import asyncio
import json
from pathlib import Path

from agent_runtime.production_fixtures import LedgerHost, bind_script
from agent_runtime.sdk_fixtures import answer
from agent_runtime.test_execution import request_value
from langchain_core.messages import AIMessage
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver

from knovum_agent_runtime.contracts.generated.control import AgentExecute
from knovum_agent_runtime.execution import Execution


def test_business_child_interrupt_replays_one_durable_intent_after_reopening(tmp_path: Path, monkeypatch) -> None:
    class ChildHost(LedgerHost):
        ready = False
        intents: set[tuple[str, str, str]] = set()

        async def request(self, method, params):
            response = await super().request(method, params)
            if method == "agentRuntime.invokeTool":
                value = self.calls[-1][1]
                self.intents.add((value["graphNamespace"], value["messageId"], value["toolCallId"]))
                if not self.ready:
                    return {"type": "await_child", "runId": "existing-child"}
                return {"type": "completed", "value": {"runId": "existing-child", "state": "succeeded"}, "media": []}
            return response

    async def run() -> None:
        scripts = {
            "main": [
                AIMessage(
                    id="business-intent",
                    content="",
                    tool_calls=[{"name": "knovum_generate_image_v1", "args": {}, "id": "business-call"}],
                    usage_metadata={"input_tokens": 10, "output_tokens": 5, "total_tokens": 15},
                ),
                answer("published", message_id="final-answer"),
            ],
            "context_summary": [answer("unused", message_id="unused")],
            "research": [],
        }
        bind_script(monkeypatch, scripts)
        value = request_value(tmp_path)
        value["tools"] = [
            {
                "name": "knovum_generate_image_v1",
                "description": "Generate an authorized image.",
                "parameters": {"type": "object", "properties": {}},
                "readOnly": False,
            }
        ]
        host = ChildHost()
        path = str(tmp_path / "checkpoint.sqlite")
        async with AsyncSqliteSaver.from_conn_string(path) as saver:
            observation = await Execution(AgentExecute.model_validate_json(json.dumps(value)), saver, host).run()
            assert observation.state.value == "interrupted"
            assert observation.interrupts == [{"type": "child_run", "runId": "existing-child"}]
            assert len(host.intents) == 1
        host.ready = True
        scripts["main"] = [answer("published", message_id="final-answer")]
        bind_script(monkeypatch, scripts)
        value["messages"] = []
        value["resume"] = True
        value["afterCheckpointId"] = observation.checkpoint_id
        value["identity"]["controlEpoch"] = 1
        async with AsyncSqliteSaver.from_conn_string(path) as saver:
            observation = await Execution(AgentExecute.model_validate_json(json.dumps(value)), saver, host).run()
            assert observation.state.value == "completed"
            assert len(host.intents) == 1
            invocations = [params for method, params in host.calls if method == "agentRuntime.invokeTool"]
            assert len(invocations) == 2
            assert invocations[0]["identity"]["controlEpoch"] == 0
            assert invocations[1]["identity"]["controlEpoch"] == 1
            assert set(host.states.values()) == {"committed"}

    asyncio.run(run())
