from __future__ import annotations

import asyncio
from pathlib import Path

import pytest
from agent_runtime.sdk_fixtures import AdmissionRecorder, answer, scripted_model
from deepagents import create_deep_agent
from deepagents.backends import StateBackend
from deepagents.middleware.summarization import SummarizationMiddleware
from langchain.agents.middleware import AgentMiddleware
from langchain.tools import ToolRuntime
from langchain_core.messages import AIMessage, HumanMessage
from langchain_core.tools import tool
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver
from langgraph.types import Command, interrupt


def test_tool_intent_is_durable_before_side_effect_and_interrupt_replays_identity(tmp_path: Path) -> None:
    async def run() -> None:
        recorder = AdmissionRecorder()
        model = scripted_model(
            [
                AIMessage(
                    content="", id="model-intent", tool_calls=[{"name": "publish", "args": {}, "id": "call-publish"}]
                ),
                answer("published", message_id="model-answer"),
            ],
            recorder,
        )
        config = {"configurable": {"thread_id": "intent-thread"}}
        observed: list[tuple[str, str]] = []
        committed: set[tuple[str, str]] = set()
        recorder.checkpoint_reader = lambda: graph.aget_state(config)
        namespaces: list[str] = []

        @tool
        async def publish(runtime: ToolRuntime[dict, dict]) -> str:
            """Publish one already authorized asset."""
            state = await graph.aget_state(config)
            message = state.values["messages"][-1]
            identity = (message.id, message.tool_calls[0]["id"])
            assert identity == ("model-intent", "call-publish")
            assert state.config["configurable"]["checkpoint_id"]
            observed.append(identity)
            namespaces.append(runtime.config["configurable"].get("checkpoint_ns", ""))
            committed.add(identity)
            approval = interrupt({"childRunId": "existing-child"})
            return str(approval)

        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            graph = create_deep_agent(
                model=model, tools=[publish], backend=StateBackend(), checkpointer=saver, context_schema=dict
            )
            result = await graph.ainvoke(
                {"messages": [HumanMessage(content="publish", id="input-1")]},
                config,
                durability="sync",
                context={"runId": "intent-run"},
            )
            assert result["__interrupt__"][0].value == {"childRunId": "existing-child"}
            assert len(model.requests) == 1

        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            graph = create_deep_agent(
                model=model, tools=[publish], backend=StateBackend(), checkpointer=saver, context_schema=dict
            )
            result = await graph.ainvoke(
                Command(resume="child finished"), config, durability="sync", context={"runId": "intent-run"}
            )
            assert result["messages"][-1].content == "published"
            assert observed == [("model-intent", "call-publish")] * 2
            assert len(committed) == 1
            assert len(set(namespaces)) == 1
            assert namespaces[0].startswith("tools:")
            assert len(model.requests) == 2
            history = [state async for state in graph.aget_state_history(config)]
            assert any(message.id == "input-1" for state in history for message in state.values.get("messages", []))
        assert len(recorder.starts) == len(recorder.completions) == 2
        assert recorder.contexts == [{"runId": "intent-run"}] * 2
        assert all(recorder.checkpoints)

    asyncio.run(run())


def test_admission_denial_prevents_model_execution() -> None:
    async def run() -> None:
        recorder = AdmissionRecorder(deny=True)
        model = scripted_model([answer("must not run", message_id="unused")], recorder)
        graph = create_deep_agent(model=model)
        with pytest.raises(PermissionError, match="admission denied"):
            await graph.ainvoke({"messages": [HumanMessage(content="hello", id="denied-input")]})
        assert model.requests == []
        assert recorder.completions == []
        assert len(recorder.starts) == 1

    asyncio.run(run())


def test_pause_is_a_durable_boundary_and_resume_does_not_append_input_twice(tmp_path: Path) -> None:
    async def run() -> None:
        model = scripted_model([answer("resumed", message_id="resumed-answer")])

        class PauseBoundary(AgentMiddleware):
            paused = True

            async def abefore_model(self, state, runtime):
                if self.paused:
                    interrupt({"reason": "pause"})
                return None

        boundary = PauseBoundary()
        config = {"configurable": {"thread_id": "paused-thread"}}
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            graph = create_deep_agent(model=model, middleware=[boundary], checkpointer=saver)
            result = await graph.ainvoke(
                {"messages": [HumanMessage(content="hello", id="accepted-input")]}, config, durability="sync"
            )
            assert result["__interrupt__"][0].value == {"reason": "pause"}
            assert model.requests == []
            state = await graph.aget_state(config)
            assert [message.id for message in state.values["messages"]] == ["accepted-input"]
            boundary.paused = False
            result = await graph.ainvoke(Command(resume=True), config, durability="sync")
            assert len([message for message in result["messages"] if message.id == "accepted-input"]) == 1
            assert result["messages"][-1].content == "resumed"

    asyncio.run(run())


def test_research_uses_its_own_admission_and_cannot_inherit_writes() -> None:
    async def run() -> None:
        main_recorder, research_recorder = AdmissionRecorder(), AdmissionRecorder()
        main = scripted_model(
            [
                AIMessage(
                    content="",
                    id="delegate",
                    tool_calls=[
                        {
                            "name": "task",
                            "args": {"description": "research", "subagent_type": "research"},
                            "id": "research-call",
                        }
                    ],
                ),
                answer("research complete", message_id="main-final"),
            ],
            main_recorder,
        )
        research = scripted_model([answer("findings", message_id="research-final")], research_recorder)

        @tool
        async def write_asset() -> str:
            """Write one authorized asset."""
            raise AssertionError("Research must not receive write capability")

        graph = create_deep_agent(
            model=main,
            tools=[write_asset],
            subagents=[{"name": "research", "description": "Read-only research", "model": research, "tools": []}],
        )
        result = await graph.ainvoke({"messages": [HumanMessage(content="research", id="research-input")]})
        assert result["messages"][-1].content == "research complete"
        assert len(main_recorder.starts) == len(main_recorder.completions) == 2
        assert len(research_recorder.starts) == len(research_recorder.completions) == 1
        assert "write_asset" not in research.advertised_tools[0]
        assert "task" not in research.advertised_tools[0]
        assert "execute" not in research.advertised_tools[0]

    asyncio.run(run())


def test_summary_requests_have_admission_and_original_answers_remain_projectable(tmp_path: Path) -> None:
    async def run() -> None:
        main_recorder, summary_recorder = AdmissionRecorder(), AdmissionRecorder()
        main = scripted_model([answer("complete answer", message_id="main-answer")], main_recorder)
        summary = scripted_model([answer("condensed history", message_id="summary-answer")], summary_recorder)
        messages = [
            HumanMessage(content="older input", id="old-input"),
            answer("older complete answer", message_id="old-answer"),
        ]
        config = {"configurable": {"thread_id": "summary-thread"}}
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            graph = create_deep_agent(
                model=main,
                backend=StateBackend(),
                checkpointer=saver,
                middleware=[
                    SummarizationMiddleware(
                        model=summary, backend=StateBackend(), trigger=("messages", 3), keep=("messages", 1)
                    )
                ],
            )
            await graph.ainvoke(
                {"messages": [*messages, HumanMessage(content="new input", id="new-input")]}, config, durability="sync"
            )
            assert len(summary_recorder.starts) == len(summary_recorder.completions) == 1
            assert len(main_recorder.starts) == len(main_recorder.completions) == 1
            history = [state async for state in graph.aget_state_history(config)]
            originals = {
                message.id: message.content for state in history for message in state.values.get("messages", [])
            }
            assert originals["old-answer"] == "older complete answer"
            assert originals["main-answer"] == "complete answer"

    asyncio.run(run())
