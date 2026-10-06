from __future__ import annotations

import asyncio
import json
import os
import sqlite3
import sys
from pathlib import Path

from agent_runtime.sdk_fixtures import answer, scripted_model
from deepagents import create_deep_agent
from langchain.tools import ToolRuntime
from langchain_core.messages import AIMessage, HumanMessage
from langchain_core.tools import tool
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver


async def run(directory: Path, crash: str) -> None:
    model = scripted_model(
        [AIMessage(content="", id="durable-intent", tool_calls=[{"name": "publish", "args": {}, "id": "durable-call"}])]
        if crash != "resume"
        else [answer("completed", message_id="durable-answer")]
    )
    config = {"configurable": {"thread_id": "crash-thread"}}

    @tool
    async def publish(runtime: ToolRuntime[dict, dict]) -> str:
        """Publish through a separate durable, idempotent host ledger."""
        snapshot = await graph.aget_state(config)
        message = snapshot.values["messages"][-1]
        identity = json.dumps([runtime.config["configurable"]["checkpoint_ns"], message.id, runtime.tool_call_id])
        if crash == "before_write":
            os._exit(86)
        with sqlite3.connect(directory / "host.sqlite") as database:
            database.execute("CREATE TABLE IF NOT EXISTS operations (id TEXT PRIMARY KEY)")
            database.execute("INSERT OR IGNORE INTO operations VALUES (?)", (identity,))
        if crash == "after_write":
            os._exit(86)
        return "receipt"

    async with AsyncSqliteSaver.from_conn_string(str(directory / "checkpoint.sqlite")) as saver:
        graph = create_deep_agent(model=model, tools=[publish], checkpointer=saver, context_schema=dict)
        inputs = None if crash == "resume" else {"messages": [HumanMessage(content="publish", id="input:crash")]}
        result = await graph.ainvoke(inputs, config, durability="sync", context={})
        assert result["messages"][-1].content == "completed"
        assert len(model.requests) == 1


if __name__ == "__main__":
    asyncio.run(run(Path(sys.argv[1]), sys.argv[2]))
