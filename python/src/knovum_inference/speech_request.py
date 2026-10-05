from __future__ import annotations

import hashlib
import math
import time
from pathlib import Path
from typing import Any
from uuid import UUID

from knovum_inference.errors import invalid_params, server_error


def milliseconds(seconds: float) -> int:
    if not math.isfinite(seconds) or seconds < 0:
        raise server_error("invalid_output", "Speech engine returned an invalid timestamp")
    return round(seconds * 1000)


def verify_audio(audio_ref: Any) -> Path:
    path = Path(audio_ref.path)
    if not path.is_absolute() or not path.is_file():
        raise invalid_params("Audio reference must name an existing absolute file")
    if path.stat().st_size != audio_ref.byte_length:
        raise server_error("stale_revision", "Audio reference length changed")
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(1024 * 1024):
            digest.update(chunk)
    if digest.hexdigest() != audio_ref.sha256:
        raise server_error("stale_revision", "Audio reference digest changed")
    return path


def check_deadline(deadline_ms: int) -> None:
    if int(time.time() * 1000) >= deadline_ms:
        raise server_error("deadline_exceeded", "Speech request deadline has elapsed")


def attempt_directory(work_directory: Path, request_id: str) -> Path:
    try:
        if str(UUID(request_id)) != request_id:
            raise ValueError("Request ID is not canonical")
    except ValueError as error:
        raise invalid_params("Speech request ID must be a canonical UUID") from error
    path = work_directory / f"attempt-{request_id}"
    try:
        path.mkdir(mode=0o700)
    except OSError as error:
        raise server_error("resource_exhausted", "Speech attempt directory cannot be created") from error
    return path
