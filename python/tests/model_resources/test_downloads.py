from __future__ import annotations

import io
import json
import sys
import types
from pathlib import Path
from unittest.mock import Mock

import pytest
from pydantic import ValidationError

from knovum_model_resources import hugging_face, model_scope, server
from knovum_model_resources.contracts import DownloadRequest
from knovum_model_resources.progress import DownloadProgress


def request_payload(directory: Path, provider: str) -> dict:
    source = {"provider": provider, "repository": "owner/model", "revision": "a" * 40}
    if provider == "hugging_face":
        source["endpoint"] = "https://huggingface.co"
    return {
        "directory": str(directory),
        "source": source,
        "files": [{"path": "model.bin", "size": 10, "sha256": "b" * 64}],
    }


def test_hugging_face_uses_public_snapshot_api_with_fixed_inventory(monkeypatch, tmp_path) -> None:
    snapshot_download = Mock()
    monkeypatch.setitem(sys.modules, "huggingface_hub", types.SimpleNamespace(snapshot_download=snapshot_download))
    request = DownloadRequest.model_validate_json(json.dumps(request_payload(tmp_path, "hugging_face")))
    output = io.StringIO()
    hugging_face.download(request, request.source, DownloadProgress(output, 10))
    arguments = snapshot_download.call_args.kwargs
    assert arguments["repo_id"] == "owner/model"
    assert arguments["revision"] == "a" * 40
    assert arguments["local_dir"] == str(tmp_path)
    assert arguments["allow_patterns"] == ["model.bin"]
    assert arguments["endpoint"] == "https://huggingface.co"
    assert arguments["token"] is False
    assert arguments["max_workers"] == 2
    diagnostics = io.StringIO()
    with arguments["tqdm_class"](total=10, unit="B", disable=True, file=diagnostics) as bar:
        bar.update(5)
    assert json.loads(output.getvalue())["downloaded"] == 5
    assert diagnostics.getvalue().strip() == ""


def test_model_scope_uses_public_callbacks_without_personal_credentials(monkeypatch, tmp_path) -> None:
    class Callback:
        def __init__(self, filename: str, file_size: int) -> None:
            self.filename = filename
            self.file_size = file_size

    api = Mock()
    factory = Mock(return_value=api)
    monkeypatch.setitem(sys.modules, "modelscope_hub", types.SimpleNamespace(HubApi=factory, ProgressCallback=Callback))
    request = DownloadRequest.model_validate_json(json.dumps(request_payload(tmp_path, "model_scope")))
    output = io.StringIO()
    model_scope.download(request, request.source, DownloadProgress(output, 10))
    factory.assert_called_once_with(endpoint="https://modelscope.cn", token="")
    arguments = api.download_repo.call_args.kwargs
    assert arguments["revision"] == "a" * 40
    assert arguments["local_dir"] == str(tmp_path)
    assert arguments["allow_patterns"] == ["model.bin"]
    callback = arguments["progress_callbacks"][0]("model.bin", 10)
    callback.update(5)
    callback.end()
    assert all(json.loads(line)["total"] == 10 for line in output.getvalue().splitlines())


@pytest.mark.parametrize("mutation", ["unknown", "provider", "files"])
def test_closed_download_request_rejects_invalid_boundaries(tmp_path, mutation) -> None:
    payload = request_payload(tmp_path, "hugging_face")
    if mutation == "unknown":
        payload["token"] = "must-not-be-accepted"
    elif mutation == "provider":
        payload["source"]["provider"] = "arbitrary_downloader"
    else:
        payload["files"] = []
    with pytest.raises(ValidationError):
        DownloadRequest.model_validate_json(json.dumps(payload))


def test_sdk_stdout_cannot_corrupt_download_protocol(monkeypatch, tmp_path) -> None:
    def download(*_):
        print("SDK diagnostic")

    monkeypatch.setattr(hugging_face, "download", download)
    monkeypatch.setattr(
        sys, "stdin", io.TextIOWrapper(io.BytesIO(json.dumps(request_payload(tmp_path, "hugging_face")).encode()))
    )
    output, diagnostics = io.StringIO(), io.StringIO()
    monkeypatch.setattr(sys, "stdout", output)
    monkeypatch.setattr(sys, "stderr", diagnostics)
    server.main()
    assert output.getvalue() == '{"event":"complete"}\n'
    assert diagnostics.getvalue() == "SDK diagnostic\n"


def test_sdk_failure_emits_one_terminal_error_without_sensitive_exception(monkeypatch, tmp_path) -> None:
    def download(*_):
        raise RuntimeError("https://user:secret@example.test")

    monkeypatch.setattr(hugging_face, "download", download)
    monkeypatch.setattr(
        sys, "stdin", io.TextIOWrapper(io.BytesIO(json.dumps(request_payload(tmp_path, "hugging_face")).encode()))
    )
    output = io.StringIO()
    monkeypatch.setattr(sys, "stdout", output)
    with pytest.raises(SystemExit, match="1"):
        server.main()
    assert json.loads(output.getvalue()) == {"event": "error", "message": "Model download failed (RuntimeError)"}
