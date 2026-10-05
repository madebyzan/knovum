from __future__ import annotations

import asyncio
import threading
from dataclasses import dataclass

from knovum_inference.speech_stream import ChunkCredits


@dataclass(slots=True)
class ActiveRequest:
    request_id: str
    attempt_epoch: int
    cancelled: threading.Event
    done: asyncio.Event
    chunk_credits: ChunkCredits | None = None
