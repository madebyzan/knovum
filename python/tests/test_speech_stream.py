from __future__ import annotations

import struct
import threading
import time
from collections.abc import Callable
from pathlib import Path

import pytest

from knovum_inference.errors import RpcError
from knovum_inference.speech_stream import MAX_UNCONSUMED_BYTES, ChunkCredits


def credits(cancelled: threading.Event | None = None) -> ChunkCredits:
    return ChunkCredits("request", 3, int(time.time() * 1000) + 5000, cancelled or threading.Event())


def blocked_reservation(window: ChunkCredits, size: int) -> tuple[threading.Event, list[str], threading.Thread]:
    entered = threading.Event()
    completed = threading.Event()
    outcomes: list[str] = []

    def reserve() -> None:
        entered.set()
        try:
            window.reserve(size)
            outcomes.append("reserved")
        except RpcError as error:
            outcomes.append(error.stable_code)
        finally:
            completed.set()

    thread = threading.Thread(target=reserve, daemon=True)
    thread.start()
    assert entered.wait(1)
    assert not completed.wait(0.03)
    return completed, outcomes, thread


def finish(completed: threading.Event, outcomes: list[str], thread: threading.Thread, expected: str) -> None:
    assert completed.wait(1), "credit waiter did not wake"
    thread.join(1)
    assert not thread.is_alive()
    assert outcomes == [expected]


def test_ninth_chunk_waits_for_verified_consumption() -> None:
    window = credits()
    for seq in range(1, 9):
        window.reserve(2)
        window.publish(seq, 2)
    completed, outcomes, thread = blocked_reservation(window, 2)
    window.acknowledge("request", 3, 1, 2)
    finish(completed, outcomes, thread, "reserved")


def test_byte_window_applies_before_file_count_limit() -> None:
    window = credits()
    window.reserve(MAX_UNCONSUMED_BYTES)
    window.publish(1, MAX_UNCONSUMED_BYTES)
    completed, outcomes, thread = blocked_reservation(window, 2)
    window.acknowledge("request", 3, 1, MAX_UNCONSUMED_BYTES)
    finish(completed, outcomes, thread, "reserved")


@pytest.mark.parametrize("ack", [("wrong", 3, 1, 2), ("request", 4, 1, 2), ("request", 3, 2, 2), ("request", 3, 1, 3)])
def test_wrong_ack_invalidates_the_stream(ack: tuple[str, int, int, int]) -> None:
    window = credits()
    window.reserve(2)
    window.publish(1, 2)
    window.acknowledge(*ack)
    with pytest.raises(RpcError) as caught:
        window.wait_until_consumed()
    assert caught.value.stable_code == "invalid_output"


def test_duplicate_ack_is_rejected() -> None:
    window = credits()
    window.reserve(2)
    window.publish(1, 2)
    window.acknowledge("request", 3, 1, 2)
    window.acknowledge("request", 3, 1, 2)
    with pytest.raises(RpcError) as caught:
        window.reserve(2)
    assert caught.value.stable_code == "invalid_output"


def test_cancel_wakes_a_worker_blocked_by_backpressure() -> None:
    cancelled = threading.Event()
    window = credits(cancelled)
    window.reserve(MAX_UNCONSUMED_BYTES)
    window.publish(1, MAX_UNCONSUMED_BYTES)
    completed, outcomes, thread = blocked_reservation(window, 2)
    cancelled.set()
    finish(completed, outcomes, thread, "cancelled")


@pytest.mark.parametrize("operation", [lambda window: window.reserve(2), lambda window: window.wait_until_consumed()])
def test_expired_request_never_reserves_or_finishes(operation: Callable[[ChunkCredits], None]) -> None:
    window = ChunkCredits("request", 3, 0, threading.Event())
    with pytest.raises(RpcError) as caught:
        operation(window)
    assert caught.value.stable_code == "deadline_exceeded"


def test_float_master_preserves_samples_outside_preview_range(tmp_path: Path) -> None:
    from knovum_inference.speech_stream import SynthesisChunkPublisher

    events: list[dict[str, object]] = []
    work = tmp_path
    request_id = "00000000-0000-0000-0000-000000000001"
    window = ChunkCredits(request_id, 3, int(time.time() * 1000) + 5000, threading.Event())
    publisher = SynthesisChunkPublisher(work, request_id, 3, int(time.time() * 1000) + 5000, window, events.append)
    publisher.publish([0.25, -0.25, 1.25], 48000, "model_chunk")
    event = events[0]
    reference = event["chunkRef"]
    assert isinstance(reference, dict)
    payload = (work / str(reference["fileName"])).read_bytes()
    assert struct.unpack("<fff", payload) == (0.25, -0.25, 1.25)
    assert reference["sampleFormat"] == "pcm_f32le"
    assert reference["byteLength"] == 12
    window.acknowledge(request_id, 3, 1, 12)
    assert publisher.finish()["sampleCount"] == 3
