from __future__ import annotations

import io
import json
from concurrent.futures import ThreadPoolExecutor
from unittest.mock import patch

from knovum_model_resources.progress import DownloadProgress


def test_concurrent_file_callbacks_aggregate_actual_bytes_and_throttle() -> None:
    output = io.StringIO()
    progress = DownloadProgress(output, 100)
    with patch("knovum_model_resources.progress.time.monotonic", side_effect=[1.0, 1.01, 1.2, 1.4]):
        progress.update_file("weights", 30)
        progress.update_file("config", 5)
        progress.update_file("weights", 60)
        progress.update_file("weights", 100)
    progress.complete()
    events = [json.loads(line) for line in output.getvalue().splitlines()]
    assert [event["downloaded"] for event in events[:-1]] == [30, 65, 100]
    assert all(event["total"] == 100 for event in events[:-1])
    assert events[-1] == {"event": "complete"}


def test_parallel_chunk_callbacks_do_not_lose_progress() -> None:
    output = io.StringIO()
    progress = DownloadProgress(output, 1000)
    with ThreadPoolExecutor(max_workers=8) as executor:
        list(executor.map(lambda _: progress.advance_file("weights", 1), range(1000)))
    with patch("knovum_model_resources.progress.time.monotonic", return_value=1e12):
        progress.advance_file("weights", 0)
    assert json.loads(output.getvalue().splitlines()[-1])["downloaded"] == 1000
