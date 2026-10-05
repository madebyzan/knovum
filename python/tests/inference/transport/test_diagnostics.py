from __future__ import annotations

import asyncio
import json
from unittest.mock import AsyncMock

import pytest

from knovum_inference.contracts.generated.diagnostics import RuntimeDiagnostic
from knovum_inference.errors import RpcError
from knovum_inference.runtime.worker import Worker
from knovum_inference.transport.diagnostics import report_request_failure
from knovum_inference.transport.protocol import Request

REQUEST_ID = "34b3e359-01fd-41c5-80e9-999d316d9aaa"


@pytest.mark.parametrize(
    "error,code",
    [
        (ValueError("secret"), "invalid_output"),
        (MemoryError("secret"), "resource_exhausted"),
        (ImportError("secret"), "runtime_mismatch"),
    ],
)
def test_worker_reports_safe_structural_failures(
    error: Exception, code: str, capsys: pytest.CaptureFixture[str]
) -> None:
    async def scenario() -> None:
        worker = Worker()
        worker._dispatch = AsyncMock(side_effect=error)
        worker._writer.error = AsyncMock()
        await worker._handle(Request(REQUEST_ID, "speech.transcribe", {"text": "private input"}))
        worker._writer.error.assert_awaited_once()
        assert worker._writer.error.call_args.args[1].stable_code == code

    asyncio.run(scenario())
    output = capsys.readouterr().err
    diagnostic = RuntimeDiagnostic.model_validate_json(output)
    value = diagnostic.model_dump(by_alias=True, mode="json")
    assert value["event"] == "requestFailed"
    assert value["code"] == code
    assert value["requestId"] == REQUEST_ID
    assert value["method"] == "speech.transcribe"
    assert value["exceptionType"] == f"builtins.{type(error).__name__}"
    assert 0 < len(value["frames"]) <= 8
    assert "secret" not in output and "private input" not in output
    assert all("/" not in frame["file"] and "\\" not in frame["file"] for frame in value["frames"])


def test_untrusted_request_identity_is_not_logged(capsys: pytest.CaptureFixture[str]) -> None:
    report_request_failure(Request("private id", "private method", {}), ValueError("private message"))
    value = json.loads(capsys.readouterr().err)
    assert value["requestId"] is None
    assert value["method"] == "unknown"
    assert "private" not in json.dumps(value)


def test_expected_rpc_failures_do_not_emit_diagnostics(capsys: pytest.CaptureFixture[str]) -> None:
    async def scenario() -> None:
        worker = Worker()
        worker._dispatch = AsyncMock(side_effect=RpcError(-32000, "Cancelled", "cancelled"))
        worker._writer.error = AsyncMock()
        await worker._handle(Request(REQUEST_ID, "speech.transcribe", {}))
        worker._writer.error.assert_awaited_once()

    asyncio.run(scenario())
    assert capsys.readouterr().err == ""
