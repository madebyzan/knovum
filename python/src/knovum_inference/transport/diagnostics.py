from __future__ import annotations

import json
import sys
import traceback
from pathlib import Path
from uuid import UUID

from knovum_inference.contracts.generated.diagnostics import (
    RuntimeDiagnostic,
    RuntimeDiagnosticEvent,
    RuntimeDiagnosticMethod,
)
from knovum_inference.transport.protocol import Request
from knovum_inference.transport.validation import report_request_error, validate_response


def report_request_failure(request: Request, error: Exception) -> None:
    try:
        method = RuntimeDiagnosticMethod(request.method)
    except ValueError:
        method = RuntimeDiagnosticMethod.unknown
    frames = [
        {"file": _symbol(Path(frame.filename).name), "function": _symbol(frame.name), "line": frame.lineno}
        for frame in traceback.extract_tb(error.__traceback__)[-8:]
        if frame.lineno is not None
    ]
    diagnostic = validate_response(
        RuntimeDiagnostic,
        {
            "event": RuntimeDiagnosticEvent.request_failed,
            "code": report_request_error(error).stable_code,
            "method": method,
            "requestId": _request_id(request.request_id),
            "exceptionType": _symbol(f"{type(error).__module__}.{type(error).__qualname__}"),
            "frames": frames,
        },
    )
    print(json.dumps(diagnostic), file=sys.stderr, flush=True)


def _symbol(value: str) -> str:
    return "".join(
        character for character in value[:128] if character.isascii() and (character.isalnum() or character in "_.<>-")
    )


def _request_id(value: str | int | None) -> str | None:
    if not isinstance(value, str):
        return None
    try:
        parsed = UUID(value)
    except ValueError:
        return None
    return str(parsed) if str(parsed) == value else None
