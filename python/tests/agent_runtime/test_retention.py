from __future__ import annotations

import asyncio
import json
from unittest.mock import patch

from langgraph.checkpoint.base import empty_checkpoint
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver

from knovum_agent_runtime.contracts.generated.control import AgentPruneThreads
from knovum_agent_runtime.retention import prune_checkpoints


def request(retained: list[str], deleted: list[str] | None = None) -> AgentPruneThreads:
    return AgentPruneThreads.model_validate_json(
        json.dumps({"threadIds": deleted or [], "retainedThreadIds": retained, "orphanBeforeMs": 1735689600000})
    )


async def checkpoint(saver: AsyncSqliteSaver, thread: str, namespace: str, timestamp: str) -> None:
    value = empty_checkpoint()
    value["ts"] = timestamp
    config = await saver.aput(
        {"configurable": {"thread_id": thread, "checkpoint_ns": namespace}},
        value,
        {"source": "loop", "step": 0, "parents": {}},
        {},
    )
    await saver.aput_writes(config, [("messages", ["pending"])], "task")


def test_retained_histories_do_not_load_bodies() -> None:
    async def run() -> None:
        async with AsyncSqliteSaver.from_conn_string(":memory:") as saver:
            for _ in range(200):
                await checkpoint(saver, "kept", "", "2020-01-01T00:00:00+00:00")
            with patch.object(saver, "aget_tuple", wraps=saver.aget_tuple) as load:
                await prune_checkpoints(saver, request(["kept"]))
            load.assert_not_called()
            async with saver.conn.execute("SELECT COUNT(*) FROM checkpoints") as cursor:
                assert await cursor.fetchone() == (200,)

    asyncio.run(run())


def test_latest_namespace_head_protects_recent_orphan_and_expired_deletion_removes_writes() -> None:
    async def run() -> None:
        async with AsyncSqliteSaver.from_conn_string(":memory:") as saver:
            for _ in range(30):
                await checkpoint(saver, "expired", "", "2020-01-01T00:00:00+00:00")
            await checkpoint(saver, "recent", "", "2020-01-01T00:00:00+00:00")
            await checkpoint(saver, "recent", "delegate", "2026-01-01T00:00:00+00:00")
            await checkpoint(saver, "explicit", "", "2026-01-01T00:00:00+00:00")
            with patch.object(saver, "aget_tuple", wraps=saver.aget_tuple) as load:
                await prune_checkpoints(saver, request(["explicit"], ["explicit"]))
            assert load.call_count == 2
            for table in ("checkpoints", "writes"):
                async with saver.conn.execute(f"SELECT DISTINCT thread_id FROM {table}") as cursor:
                    assert await cursor.fetchall() == [("recent",)]

    asyncio.run(run())


def test_orphan_inspection_is_bounded_and_progresses_across_calls() -> None:
    async def run() -> None:
        async with AsyncSqliteSaver.from_conn_string(":memory:") as saver:
            for index in range(101):
                await checkpoint(saver, f"orphan:{index}", "", "2020-01-01T00:00:00+00:00")
            with patch.object(saver, "aget_tuple", wraps=saver.aget_tuple) as load:
                await prune_checkpoints(saver, request([]))
                assert load.call_count == 100
                await prune_checkpoints(saver, request([]))
                assert load.call_count == 101
            async with saver.conn.execute("SELECT COUNT(*) FROM checkpoints") as cursor:
                assert await cursor.fetchone() == (0,)

    asyncio.run(run())
