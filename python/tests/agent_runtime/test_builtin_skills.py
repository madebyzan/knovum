from __future__ import annotations

import asyncio
import json
from pathlib import Path

import pytest
from agent_runtime.production_fixtures import LedgerHost, bind_script
from agent_runtime.sdk_fixtures import answer
from agent_runtime.test_execution import request_value
from agent_runtime.test_skills import resource
from langchain_core.messages import AIMessage
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver

from knovum_agent_runtime.contracts.generated.control import AgentExecute
from knovum_agent_runtime.execution import Execution


@pytest.mark.parametrize(
    "skill_name",
    ["knovum-knowledge-v1", "knovum-evidence-brief", "knovum-meeting-notes", "knovum-document-delivery"],
)
def test_bundled_skills_are_discovered_and_read_by_the_production_graph(
    tmp_path: Path, monkeypatch, skill_name: str
) -> None:
    async def run() -> None:
        directory = Path(__file__).resolve().parents[3] / "src-tauri/resources/agent-plugins/knowledge-work/skills"
        files = [
            resource(f"/skills/{path.parent.name}/SKILL.md", path.read_bytes(), False)
            for path in sorted(directory.glob("*/SKILL.md"))
        ]
        assert len(files) == 4
        selected = next(file for file in files if file.path == f"/skills/{skill_name}/SKILL.md")
        models = bind_script(
            monkeypatch,
            {
                "main": [
                    AIMessage(
                        content="",
                        id="skill-read",
                        usage_metadata={"input_tokens": 10, "output_tokens": 5, "total_tokens": 15},
                        tool_calls=[
                            {"name": "read_file", "args": {"file_path": selected.path}, "id": "skill-read-call"}
                        ],
                    ),
                    answer("Instructions loaded.", message_id="skill-answer"),
                ],
                "context_summary": [],
                "research": [],
            },
        )
        value = request_value()
        value["skills"] = [file.model_dump(mode="json", by_alias=True) for file in files]
        host = LedgerHost()
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            observation = await Execution(AgentExecute.model_validate_json(json.dumps(value)), saver, host).run()
            assert observation.state.value == "completed"
            requests = models["main"].requests
            assert len(requests) == 2
            system_prompt = str(requests[0][0].content)
            assert all(file.path in system_prompt for file in files), system_prompt
            tool_reply = next(message for message in requests[1] if message.type == "tool")
            heading = next(line for line in selected.content.splitlines() if line.startswith("# "))
            assert heading in str(tool_reply.content)
            assert set(host.states.values()) == {"committed"}

    asyncio.run(run())
