from __future__ import annotations

from collections.abc import AsyncIterator

from langgraph.checkpoint.base import CheckpointTuple
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver


async def latest_namespace_checkpoints(saver: AsyncSqliteSaver, thread_id: str) -> AsyncIterator[CheckpointTuple]:
    """Select namespace identities; the SDK loads each latest checkpoint and its pending writes."""
    await saver.setup()
    async with (
        saver.lock,
        saver.conn.execute(
            "SELECT checkpoint_ns, MAX(checkpoint_id) FROM checkpoints WHERE thread_id=? GROUP BY checkpoint_ns",
            (thread_id,),
        ) as cursor,
    ):
        identities = await cursor.fetchall()
    for namespace, checkpoint_id in identities:
        checkpoint = await saver.aget_tuple(
            {"configurable": {"thread_id": thread_id, "checkpoint_ns": namespace, "checkpoint_id": checkpoint_id}}
        )
        if checkpoint is None:
            raise ValueError("agent-runtime-checkpoint-not-found")
        yield checkpoint
