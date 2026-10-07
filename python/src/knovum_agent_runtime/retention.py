from __future__ import annotations

import json
from datetime import datetime

from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver

from .contracts.generated.control import AgentPruneThreads

ORPHAN_INSPECTION_LIMIT = 100


async def prune_checkpoints(saver: AsyncSqliteSaver, request: AgentPruneThreads) -> None:
    """Inspect one latest SDK checkpoint per orphan, never retained history bodies."""
    await saver.setup()
    async with (
        saver.lock,
        saver.conn.execute(
            """WITH orphan_heads AS (
                 SELECT thread_id, MAX(checkpoint_id) AS checkpoint_id FROM checkpoints
                 WHERE thread_id NOT IN (SELECT value FROM json_each(?)) GROUP BY thread_id
               ) SELECT c.thread_id, MIN(c.checkpoint_ns), c.checkpoint_id
                 FROM orphan_heads h JOIN checkpoints c
                   ON c.thread_id=h.thread_id AND c.checkpoint_id=h.checkpoint_id
                 GROUP BY c.thread_id,c.checkpoint_id ORDER BY c.checkpoint_id,c.thread_id LIMIT ?""",
            (json.dumps(request.retained_thread_ids), ORPHAN_INSPECTION_LIMIT),
        ) as cursor,
    ):
        identities = await cursor.fetchall()
    expired = set(request.thread_ids)
    for thread_id, namespace, checkpoint_id in identities:
        checkpoint = await saver.aget_tuple(
            {"configurable": {"thread_id": thread_id, "checkpoint_ns": namespace, "checkpoint_id": checkpoint_id}}
        )
        if checkpoint is None:
            raise ValueError("agent-runtime-checkpoint-not-found")
        timestamp = datetime.fromisoformat(checkpoint.checkpoint["ts"]).timestamp() * 1000
        if timestamp < request.orphan_before_ms:
            expired.add(thread_id)
    for thread_id in expired:
        await saver.adelete_thread(thread_id)
