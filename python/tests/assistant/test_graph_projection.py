from __future__ import annotations

import asyncio
import json
from pathlib import Path

import pytest
from assistant.production_fixtures import LedgerHost, bind_script
from assistant.sdk_fixtures import answer
from assistant.test_execution import request_value
from langchain_core.messages import AIMessage
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver

from knovum_assistant.contracts.generated.control import AssistantExecute
from knovum_assistant.execution import Execution
from knovum_assistant.transport import HostRejected


def test_summary_receipts_commit_and_old_main_answers_remain_unique(tmp_path: Path, monkeypatch) -> None:
    async def run() -> None:
        models = bind_script(
            monkeypatch,
            {
                "main": [answer("new answer", message_id="new-answer")],
                "context_summary": [answer("condensed", message_id="summary-answer")],
            },
        )
        value = request_value()
        value["model"]["maxInputTokens"] = 8192
        value["messages"] = [
            {"id": "history:old-input", "role": "user", "content": [{"type": "text", "text": "older " * 20000}]},
            {"id": "history:old-answer", "role": "assistant", "content": [{"type": "text", "text": "previous answer"}]},
            *value["messages"],
        ]
        host = LedgerHost()
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            result = await Execution(AssistantExecute.model_validate_json(json.dumps(value)), saver, host).run()
            assert result.state.value == "completed"
            purposes = [value["purpose"] for method, value in host.calls if method == "assistant.admitModel"]
            assert purposes == ["context_summary", "main"]
            assert set(host.states.values()) == {"committed"}
            assert len(models["context_summary"].requests) == len(models["main"].requests) == 1
            projected = [
                message["text"]
                for method, value in host.calls
                if method == "assistant.commitProjection"
                for message in value["messages"]
            ]
            assert projected == ["new answer"]
            assert "summary-answer" not in {value["messageId"] for _, value in host.notifications}
        assert b"test-only" not in (tmp_path / "checkpoint.sqlite").read_bytes()

    asyncio.run(run())


def test_research_receipts_reconcile_in_their_durable_scope_and_cannot_delegate(tmp_path: Path, monkeypatch) -> None:
    async def run() -> None:
        models = bind_script(
            monkeypatch,
            {
                "main": [
                    AIMessage(
                        content="",
                        id="delegate",
                        usage_metadata={"input_tokens": 10, "output_tokens": 5, "total_tokens": 15},
                        tool_calls=[
                            {
                                "name": "task",
                                "args": {"description": "read evidence", "subagent_type": "research"},
                                "id": "research-call",
                            }
                        ],
                    ),
                    answer("root answer", message_id="root-answer"),
                ],
                "context_summary": [answer("unused summary", message_id="unused")],
                "research": [answer("read-only findings", message_id="research-answer")],
            },
        )
        value = request_value()
        value["research"] = True
        host = LedgerHost()
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            result = await Execution(AssistantExecute.model_validate_json(json.dumps(value)), saver, host).run()
            assert result.state.value == "completed"
            assert set(host.states.values()) == {"committed"}
            assert "task" not in models["research"].advertised_tools[0]
            assert "write_file" not in models["research"].advertised_tools[0]
            assert "execute" not in models["research"].advertised_tools[0]
            scopes = [
                value["graphNamespace"]
                for method, value in host.calls
                if method == "assistant.admitModel" and value["purpose"] == "research"
            ]
            assert len(scopes) == 1 and "|" in scopes[0]
            projected = [
                message["text"]
                for method, value in host.calls
                if method == "assistant.commitProjection"
                for message in value["messages"]
            ]
            assert projected == ["root answer"]

    asyncio.run(run())


def test_rejected_recovery_releases_the_execution_slot(tmp_path: Path, monkeypatch) -> None:
    async def run() -> None:
        bind_script(
            monkeypatch,
            {
                "main": [answer("unused", message_id="unused")],
                "context_summary": [answer("unused", message_id="unused")],
            },
        )
        host = LedgerHost()
        host.states["lost-request"] = "unknown"
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            execution = Execution(AssistantExecute.model_validate_json(json.dumps(request_value())), saver, host)
            with pytest.raises(HostRejected, match="model-outcome-unknown"):
                await execution.run()
            assert execution.task is None
            assert not any(method == "assistant.admitModel" for method, _ in host.calls)

    asyncio.run(run())


def test_plan_mode_publishes_sdk_todos_through_the_product_projection(tmp_path: Path, monkeypatch) -> None:
    async def run() -> None:
        bind_script(
            monkeypatch,
            {
                "main": [
                    AIMessage(
                        id="plan-intent",
                        content="",
                        usage_metadata={"input_tokens": 10, "output_tokens": 5, "total_tokens": 15},
                        tool_calls=[
                            {
                                "name": "write_todos",
                                "args": {"todos": [{"content": "Read authorized sources", "status": "pending"}]},
                                "id": "plan-call",
                            }
                        ],
                    ),
                    answer("The plan is ready.", message_id="plan-answer"),
                ],
                "context_summary": [answer("unused", message_id="unused")],
            },
        )
        value = request_value()
        value["plan"] = True
        host = LedgerHost()
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            result = await Execution(AssistantExecute.model_validate_json(json.dumps(value)), saver, host).run()
            assert result.state.value == "completed"
            projections = [value for method, value in host.calls if method == "assistant.commitProjection"]
            assert projections[-1]["plan"] == [{"content": "Read authorized sources", "state": "pending"}]
            assert set(host.states.values()) == {"committed"}

    asyncio.run(run())
