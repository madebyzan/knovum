from __future__ import annotations

import os
import tarfile
from pathlib import Path
from unittest.mock import Mock

import pytest

from runtime.artifacts import archive_runtime, materialize_aliases
from runtime.signing import sign_runtime, verify_runtime


def test_runtime_archive_never_conveys_symbolic_links(tmp_path: Path) -> None:
    root = tmp_path / "runtime"
    root.mkdir()
    (root / "reference").write_bytes(b"compiled")
    (root / "alias").symlink_to("reference")
    with pytest.raises(ValueError, match="nonregular"):
        archive_runtime(root, tmp_path / "runtime.tar.gz")


def test_windows_compiled_extensions_are_signed_and_verified(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    for name in ("python.exe", "lib.dll", "extension.pyd"):
        (tmp_path / name).write_bytes(b"MZ\0\0binary")
    (tmp_path / "worker.py").write_text("pass\n")
    monkeypatch.setenv("KNOVUM_WINDOWS_SIGNTOOL", "controlled-signtool")
    monkeypatch.setenv("KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT", "A" * 40)
    monkeypatch.setenv("KNOVUM_WINDOWS_TIMESTAMP_URL", "https://timestamp.example")
    commands = Mock(return_value=Mock(stdout="A" * 40))
    monkeypatch.setattr("runtime.signing.subprocess.run", commands)
    signing_evidence = sign_runtime(tmp_path, "windows-x64", "production")
    assert signing_evidence["compiledBinaryCount"] == 3
    signed = [Path(call.args[0][-1]).name for call in commands.call_args_list if call.args[0][1] == "sign"]
    assert set(signed) == {"python.exe", "lib.dll", "extension.pyd"}
    commands.reset_mock()
    assert verify_runtime(tmp_path, "windows-x64") == 3
    verified = [call for call in commands.call_args_list if call.args[0][0] == "controlled-signtool"]
    assert len(verified) == 3
    assert all(call.args[0][1:4] == ["verify", "/pa", "/all"] for call in verified)


def test_qa_signing_modes_cannot_impersonate_other_platforms(tmp_path: Path) -> None:
    (tmp_path / "python.exe").write_bytes(b"MZ\0\0")
    with pytest.raises(ValueError, match="Windows"):
        sign_runtime(tmp_path, "windows-x64", "adhoc")
    (tmp_path / "python").write_bytes(b"\xcf\xfa\xed\xfe")
    with pytest.raises(ValueError, match="macOS"):
        sign_runtime(tmp_path, "macos-arm64", "unsigned")


def test_runtime_archive_materializes_hardlinked_files(tmp_path: Path) -> None:
    root = tmp_path / "runtime"
    root.mkdir()
    (root / "compiled").write_bytes(b"binary")
    os.link(root / "compiled", root / "alias")
    output = tmp_path / "runtime.tar.gz"
    archive_runtime(root, output)
    with tarfile.open(output) as archive_file:
        entries = archive_file.getmembers()
        assert len(entries) == 2
        assert all(entry.isfile() for entry in entries)
        contents = []
        for entry in entries:
            extracted = archive_file.extractfile(entry)
            assert extracted is not None
            with extracted:
                contents.append(extracted.read())
        assert set(contents) == {b"binary"}


def test_windows_valid_signature_from_wrong_signer_is_rejected(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    (tmp_path / "python.exe").write_bytes(b"MZ\0\0")
    monkeypatch.setenv("KNOVUM_WINDOWS_SIGNTOOL", "controlled-signtool")
    monkeypatch.setenv("KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT", "A" * 40)
    monkeypatch.setattr(
        "runtime.signing.subprocess.run",
        Mock(return_value=Mock(stdout="B" * 40)),
    )
    with pytest.raises(ValueError, match="signer differs"):
        verify_runtime(tmp_path, "windows-x64")


@pytest.mark.parametrize("target", ["macos-arm64", "macos-x64"])
def test_mac_signature_requirement_binds_approved_identity_and_team(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, target: str
) -> None:
    (tmp_path / "python").write_bytes(b"\xcf\xfa\xed\xfe")
    identity = "Developer ID Application: Example Company (ABCDEFGHIJ)"
    monkeypatch.setenv("APPLE_SIGNING_IDENTITY", identity)
    commands = Mock()
    monkeypatch.setattr("runtime.signing.subprocess.run", commands)
    verify_runtime(tmp_path, target)
    command = commands.call_args.args[0]
    assert command[:4] == ["codesign", "--verify", "--strict", "-R"]
    assert 'certificate leaf[subject.CN] = "' + identity + '"' in command[4]
    assert 'certificate leaf[subject.OU] = "ABCDEFGHIJ"' in command[4]


def test_internal_directory_aliases_are_materialized_without_links(tmp_path: Path) -> None:
    root = tmp_path / "runtime"
    (root / "lib/python3.11").mkdir(parents=True)
    (root / "lib/python3.11/module.py").write_text("module")
    (root / "lib/python3.1").symlink_to("python3.11", target_is_directory=True)
    materialize_aliases(root)
    assert (root / "lib/python3.1/module.py").read_text() == "module"
    assert not any(path.is_symlink() for path in root.rglob("*"))


def test_directory_alias_cycle_is_rejected(tmp_path: Path) -> None:
    root = tmp_path / "runtime"
    (root / "lib").mkdir(parents=True)
    (root / "lib/cycle").symlink_to("..", target_is_directory=True)
    with pytest.raises(ValueError, match="cycle"):
        materialize_aliases(root)


def test_external_directory_alias_is_rejected(tmp_path: Path) -> None:
    root = tmp_path / "runtime"
    root.mkdir()
    (root / "outside").symlink_to(tmp_path, target_is_directory=True)
    with pytest.raises(ValueError, match="internal"):
        materialize_aliases(root)
