from __future__ import annotations

import asyncio
import threading

from knovum_inference.transport.input_stream import MAX_BUFFERED_MESSAGES, InputStream


def test_input_stream_backpressures_reader_until_capacity_is_available() -> None:
    async def scenario() -> None:
        stream = InputStream(asyncio.get_running_loop())
        for index in range(MAX_BUFFERED_MESSAGES):
            await stream._queue.put(str(index).encode())

        delivered: list[bool] = []
        thread = threading.Thread(target=lambda: delivered.append(stream._deliver(b"next")))
        thread.start()
        await asyncio.sleep(0.05)
        assert thread.is_alive()

        await stream.receive()
        await asyncio.to_thread(thread.join, 1)
        stream.close()
        assert not thread.is_alive()
        assert delivered == [True]

    asyncio.run(scenario())


def test_input_stream_close_cancels_a_reader_blocked_on_delivery() -> None:
    async def scenario() -> None:
        stream = InputStream(asyncio.get_running_loop())
        for index in range(MAX_BUFFERED_MESSAGES):
            await stream._queue.put(str(index).encode())

        delivered: list[bool] = []
        thread = threading.Thread(target=lambda: delivered.append(stream._deliver(b"blocked")))
        thread.start()
        await asyncio.sleep(0.05)
        assert thread.is_alive()

        stream.close()
        await asyncio.to_thread(thread.join, 1)
        assert not thread.is_alive()
        assert delivered == [False]

    asyncio.run(scenario())
