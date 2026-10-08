from __future__ import annotations

import json
from pathlib import Path
from unittest.mock import Mock

import pytest

from runtime import build_python


def test_development_mode_does_not_require_a_publication_url() -> None:
    args = build_python.arguments(["--profile", "resources", "--output", "/tmp/runtime", "--development"])
    assert args.development
    assert args.archive_url is None


def test_development_and_publication_are_exclusive() -> None:
    with pytest.raises(SystemExit):
        build_python.arguments(
            [
                "--profile",
                "resources",
                "--output",
                "/tmp/runtime",
                "--development",
                "--archive-url",
                "https://example.invalid/runtime.tar.gz",
                "--signing",
                "adhoc",
            ]
        )


@pytest.mark.parametrize("fail_activation", [False, True])
def test_development_producer_activates_runtime_and_evidence_without_archiving(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    fail_activation: bool,
) -> None:
    output = tmp_path / "development runtime 中文"
    interpreter = tmp_path / "interpreter"
    interpreter.mkdir()
    (interpreter / "runtime-manifest.json").write_text("{}")
    monkeypatch.setattr(build_python, "host_target", lambda: "macos-x64")
    monkeypatch.setattr(build_python, "install_conda_packages", lambda *_: (interpreter, {}))
    monkeypatch.setattr(build_python, "sign_runtime", lambda *_: {"mode": "adhoc", "compiledBinaryCount": 1})
    smoke = Mock(side_effect=[None, RuntimeError("final smoke failed")] if fail_activation else None)
    monkeypatch.setattr(build_python, "relocated_smoke", smoke)
    archived = Mock()
    monkeypatch.setattr(build_python, "archive_runtime", archived)

    def dependencies(_runtime: Path, scratch: Path, *_args: object) -> dict[str, str]:
        (scratch / "build-evidence").mkdir()
        return {}

    monkeypatch.setattr(build_python, "install_dependencies", dependencies)

    def run(*args: str, **_kwargs: object) -> str:
        if args[:2] == ("uv", "--version"):
            return "uv 0.12.23"
        if args[:3] == ("git", "rev-parse", "HEAD"):
            return "a" * 40
        return ""

    monkeypatch.setattr(build_python, "run", run)
    if fail_activation:
        with pytest.raises(RuntimeError, match="final smoke failed"):
            build_python.build(output, "adhoc", None, "resources")
        assert not output.exists()
        return
    build_python.build(output, "adhoc", None, "resources")
    assert (output / "runtime/runtime-manifest.json").is_file()
    record = json.loads((output / "build-evidence/producer.json").read_text())
    assert record["profile"] == "resources"
    archived.assert_not_called()
    assert not (output / "runtime-descriptor.json").exists()
    assert smoke.call_args.args[0] == output / "runtime"


def test_existing_destination_is_never_overwritten(tmp_path: Path) -> None:
    (tmp_path / "keep.txt").write_text("keep")
    with pytest.raises(ValueError, match="output already exists"):
        build_python.build(tmp_path, "adhoc", None, "resources")
    assert (tmp_path / "keep.txt").read_text() == "keep"
