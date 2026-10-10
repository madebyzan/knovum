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
    [
        "knovum-knowledge-work",
        "knovum-evidence-brief",
        "knovum-meeting-notes",
        "knovum-document-delivery",
        "knovum-study-companion",
    ],
)
def test_bundled_skills_are_discovered_and_read_by_the_production_graph(
    tmp_path: Path, monkeypatch, skill_name: str
) -> None:
    async def run() -> None:
        directory = Path(__file__).resolve().parents[3] / "src-tauri/resources/agent-plugins"
        files = [
            resource(f"/skills/{path.relative_to(package / 'skills').as_posix()}", path.read_bytes(), False)
            for package in sorted(directory.iterdir())
            for path in sorted((package / "skills").rglob("*"))
            if path.is_file()
        ]
        instructions = [file for file in files if file.path.endswith("/SKILL.md")]
        assert len(instructions) == 8
        selected = next(file for file in instructions if file.path == f"/skills/{skill_name}/SKILL.md")
        reference = next(file for file in files if file.path.startswith(f"/skills/{skill_name}/references/"))
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
                    AIMessage(
                        content="",
                        id="reference-read",
                        usage_metadata={"input_tokens": 10, "output_tokens": 5, "total_tokens": 15},
                        tool_calls=[
                            {"name": "read_file", "args": {"file_path": reference.path}, "id": "reference-read-call"}
                        ],
                    ),
                    answer("Instructions loaded.", message_id="skill-answer"),
                ],
                "context_summary": [],
                "research": [],
            },
        )
        request = request_value()
        request["skills"] = [file.model_dump(mode="json", by_alias=True) for file in files]
        request["skillToolBindings"] = {"github": [], "notion": [], "linear": []}
        host = LedgerHost()
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            observation = await Execution(AgentExecute.model_validate_json(json.dumps(request)), saver, host).run()
            assert observation.state.value == "completed"
            requests = models["main"].requests
            assert len(requests) == 3
            system_prompt = str(requests[0][0].content)
            assert all(file.path in system_prompt for file in instructions), system_prompt
            assert reference.content.strip() not in system_prompt
            heading = next(line for line in selected.content.splitlines() if line.startswith("# "))
            assert heading not in system_prompt
            assert reference.content.strip() not in str(requests[1])
            tool_reply = next(message for message in requests[1] if message.type == "tool")
            assert heading in str(tool_reply.content)
            reference_reply = next(message for message in reversed(requests[2]) if message.type == "tool")
            reference_heading = next(line for line in reference.content.splitlines() if line.startswith("# "))
            assert reference_heading in str(reference_reply.content)
            assert set(host.states.values()) == {"committed"}

    asyncio.run(run())
