from __future__ import annotations

import asyncio
from pathlib import Path
from unittest.mock import patch

from langgraph.checkpoint.base import empty_checkpoint
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver

from knovum_agent_runtime.checkpoints import latest_namespace_checkpoints


def test_loads_only_latest_bodies_and_preserves_pending_writes(tmp_path: Path) -> None:
    async def run() -> None:
        async with AsyncSqliteSaver.from_conn_string(str(tmp_path / "checkpoint.sqlite")) as saver:
            for namespace in ["", "research"]:
                config = {"configurable": {"thread_id": "thread", "checkpoint_ns": namespace}}
                for step in range(50):
                    checkpoint = empty_checkpoint()
                    checkpoint["channel_values"] = {"model_receipts": [f"receipt:{namespace}:{step}"]}
                    config = await saver.aput(config, checkpoint, {"source": "loop", "step": step, "parents": {}}, {})
                await saver.aput_writes(config, [("model_receipts", [f"pending:{namespace}"])], "model")
            with patch.object(saver, "aget_tuple", wraps=saver.aget_tuple) as load:
                values = [value async for value in latest_namespace_checkpoints(saver, "thread")]
            assert load.await_count == 2
            assert {value.config["configurable"]["checkpoint_ns"] for value in values} == {"", "research"}
            for value in values:
                namespace = value.config["configurable"]["checkpoint_ns"]
                assert value.checkpoint["channel_values"]["model_receipts"] == [f"receipt:{namespace}:49"]
                assert value.pending_writes == [("model", "model_receipts", [f"pending:{namespace}"])]

    asyncio.run(run())
