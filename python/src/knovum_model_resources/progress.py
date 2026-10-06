from __future__ import annotations

import json
import threading
import time
from typing import TextIO


class DownloadProgress:
    def __init__(self, output: TextIO, total: int) -> None:
        self._output = output
        self._total = total
        self._lock = threading.Lock()
        self._last_update = 0.0
        self._files: dict[str, int] = {}

    def update(self, downloaded: int) -> None:
        with self._lock:
            self._publish(downloaded)

    def update_file(self, filename: str, downloaded: int) -> None:
        with self._lock:
            self._files[filename] = downloaded
            self._publish(sum(self._files.values()))

    def advance_file(self, filename: str, size: int) -> None:
        with self._lock:
            self._files[filename] = self._files.get(filename, 0) + size
            self._publish(sum(self._files.values()))

    def _publish(self, downloaded: int) -> None:
        now = time.monotonic()
        if now - self._last_update < 0.1:
            return
        self._last_update = now
        self._write({"event": "progress", "downloaded": min(downloaded, self._total), "total": self._total})

    def complete(self) -> None:
        with self._lock:
            self._write({"event": "complete"})

    def _write(self, value: dict[str, str | int]) -> None:
        self._output.write(json.dumps(value, separators=(",", ":")) + "\n")
        self._output.flush()
