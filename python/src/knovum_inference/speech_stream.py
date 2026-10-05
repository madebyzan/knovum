from __future__ import annotations

import hashlib
import os
import threading
from collections.abc import Callable, Iterable
from pathlib import Path
from typing import Any, Literal

from knovum_inference.errors import server_error
from knovum_inference.speech_common import attempt_directory, check_deadline

MAX_UNCONSUMED_FILES = 8
MAX_UNCONSUMED_BYTES = 16 * 1024 * 1024
MAX_SYNTHESIZED_AUDIO_BYTES = 32 * 1024 * 1024


class ChunkCredits:
    def __init__(
        self,
        request_id: str,
        attempt_epoch: int,
        deadline_ms: int,
        cancelled: threading.Event,
    ) -> None:
        self._request_id = request_id
        self._attempt_epoch = attempt_epoch
        self._deadline_ms = deadline_ms
        self._cancelled = cancelled
        self._condition = threading.Condition()
        self._outstanding: dict[int, int] = {}
        self._reserved_bytes = 0
        self._failure: str | None = None

    def reserve(self, byte_length: int) -> None:
        if not 0 < byte_length <= MAX_UNCONSUMED_BYTES:
            raise server_error("resource_exhausted", "Synthesis chunk exceeds the credit window")
        with self._condition:
            while (
                len(self._outstanding) >= MAX_UNCONSUMED_FILES
                or self._reserved_bytes + byte_length > MAX_UNCONSUMED_BYTES
            ):
                self._check_state()
                self._condition.wait(timeout=0.1)
            self._check_state()
            self._reserved_bytes += byte_length

    def publish(self, seq: int, byte_length: int) -> None:
        with self._condition:
            if seq in self._outstanding:
                self._failure = "Synthesis chunk sequence was reused"
                self._condition.notify_all()
                raise server_error("invalid_output", self._failure)
            self._outstanding[seq] = byte_length

    def release_reservation(self, byte_length: int) -> None:
        with self._condition:
            self._reserved_bytes -= byte_length
            self._condition.notify_all()

    def acknowledge(self, request_id: str, attempt_epoch: int, seq: int, consumed_bytes: int) -> None:
        with self._condition:
            expected = self._outstanding.get(seq)
            if (
                request_id != self._request_id
                or attempt_epoch != self._attempt_epoch
                or expected is None
                or consumed_bytes != expected
            ):
                self._failure = "Runtime acknowledged a different synthesis chunk"
            else:
                del self._outstanding[seq]
                self._reserved_bytes -= consumed_bytes
            self._condition.notify_all()

    def wait_until_consumed(self) -> None:
        with self._condition:
            while self._outstanding:
                self._check_state()
                self._condition.wait(timeout=0.1)
            self._check_state()

    def _check_state(self) -> None:
        if self._failure is not None:
            raise server_error("invalid_output", self._failure)
        if self._cancelled.is_set():
            raise server_error("cancelled", "Synthesis was cancelled")
        check_deadline(self._deadline_ms)


class SynthesisChunkPublisher:
    def __init__(
        self,
        work_directory: Path,
        request_id: str,
        attempt_epoch: int,
        deadline_ms: int,
        credits: ChunkCredits,
        notify: Callable[[dict[str, Any]], None],
    ) -> None:
        self._directory = attempt_directory(work_directory, request_id)
        self._request_id = request_id
        self._attempt_epoch = attempt_epoch
        self._deadline_ms = deadline_ms
        self._credits = credits
        self._notify = notify
        self._seq = 0
        self._byte_length = 0
        self._sample_count = 0
        self._sample_rate: int | None = None
        self._digest = hashlib.sha256()

    def publish(
        self,
        samples: Iterable[float],
        sample_rate: int,
        granularity: Literal["model_chunk", "sentence"],
    ) -> None:
        import numpy as np

        check_deadline(self._deadline_ms)
        values = np.asarray(samples, dtype=np.float32).reshape(-1)
        if values.size == 0 or not np.isfinite(values).all():
            raise server_error("invalid_output", "Synthesis returned empty or non-finite audio")
        if sample_rate <= 0 or self._sample_rate not in {None, sample_rate}:
            raise server_error("invalid_output", "Synthesis chunk sample rate differs")
        pcm = values.astype("<f4", copy=False).tobytes()
        byte_length = len(pcm)
        if self._byte_length + byte_length + 44 > MAX_SYNTHESIZED_AUDIO_BYTES:
            raise server_error("resource_exhausted", "Synthesized audio exceeds its byte limit")
        self._credits.reserve(byte_length)
        self._seq += 1
        file_name = f"chunk-{self._seq:08}.pcm"
        final = self._directory / file_name
        temporary = self._directory / f".{file_name}.part"
        try:
            with temporary.open("xb") as output:
                output.write(pcm)
                output.flush()
                os.fsync(output.fileno())
            os.replace(temporary, final)
            self._credits.publish(self._seq, byte_length)
        except Exception:
            temporary.unlink(missing_ok=True)
            final.unlink(missing_ok=True)
            self._credits.release_reservation(byte_length)
            raise
        digest = hashlib.sha256(pcm).hexdigest()
        self._notify(
            {
                "requestId": self._request_id,
                "attemptEpoch": self._attempt_epoch,
                "seq": self._seq,
                "granularity": granularity,
                "chunkRef": {
                    "fileName": f"{self._directory.name}/{file_name}",
                    "mediaType": "application/vnd.knovum.pcm-f32le",
                    "byteLength": byte_length,
                    "sha256": digest,
                    "sampleRate": sample_rate,
                    "channels": 1,
                    "sampleFormat": "pcm_f32le",
                },
                "sampleCount": len(pcm) // 4,
            }
        )
        self._digest.update(pcm)
        self._byte_length += byte_length
        self._sample_count += len(pcm) // 4
        self._sample_rate = sample_rate

    def finish(self) -> dict[str, Any]:
        self._credits.wait_until_consumed()
        if self._sample_rate is None or self._sample_count == 0:
            raise server_error("invalid_output", "Synthesis returned no audio chunks")
        return {
            "requestId": self._request_id,
            "attemptEpoch": self._attempt_epoch,
            "byteLength": self._byte_length,
            "sha256": self._digest.hexdigest(),
            "sampleRate": self._sample_rate,
            "channels": 1,
            "sampleFormat": "pcm_f32le",
            "sampleCount": self._sample_count,
        }
