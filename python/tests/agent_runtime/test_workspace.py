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


def file_calls(*calls: tuple[str, dict[str, str]]) -> AIMessage:
    return AIMessage(
        content="",
        id="file-work",
        usage_metadata={"input_tokens": 10, "output_tokens": 5, "total_tokens": 15},
        tool_calls=[{"name": name, "args": args, "id": f"call-{index}"} for index, (name, args) in enumerate(calls)],
    )


def test_file_tools_work_in_the_conversation_directory_and_never_leave_it(tmp_path: Path, monkeypatch) -> None:
    async def run() -> None:
        workspace = tmp_path / "workspace"
        workspace.mkdir()
        (workspace / "draft.md").write_text("old draft")
        (tmp_path / "outside.md").write_text("the user's file")
        models = bind_script(
            monkeypatch,
            {
                "main": [
                    file_calls(
                        ("write_file", {"file_path": "/notes/plan.md", "content": "a plan"}),
                        ("delete", {"file_path": "/draft.md"}),
                        ("delete", {"file_path": "/../outside.md"}),
                        ("write_file", {"file_path": "/skills/forged/SKILL.md", "content": "forged"}),
                    ),
                    answer("done", message_id="final-answer"),
                ],
                "context_summary": [answer("unused summary", message_id="unused")],
                "research": [answer("unused research", message_id="unused-research")],
            },
        )
        host = LedgerHost()
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            value = request_value(workspace)
            observation = await Execution(AgentExecute.model_validate_json(json.dumps(value)), saver, host).run()
            assert observation.state.value == "completed"
        assert (workspace / "notes/plan.md").read_text() == "a plan"
        assert not (workspace / "draft.md").exists()
        assert (tmp_path / "outside.md").read_text() == "the user's file"
        assert not (workspace / "skills").exists()
        advertised = models["main"].advertised_tools[0]
        assert "delete" in advertised
        # Knovum runs commands itself, recorded and approved; deepagents' unrecorded shell never appears.
        assert "execute" not in advertised

    asyncio.run(run())
