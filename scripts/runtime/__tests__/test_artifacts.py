from __future__ import annotations

import io
import os
import tarfile
from pathlib import Path
from unittest.mock import Mock

import pytest

from runtime.artifacts import archive_runtime, extract_python
from runtime.signing import sign_runtime, verify_runtime


def archive(path: Path, entries: list[tuple[str, bytes | str]]) -> None:
    with tarfile.open(path, "w") as output:
        for name, contents in entries:
            info = tarfile.TarInfo(name)
            if isinstance(contents, str):
                info.type = tarfile.SYMTYPE
                info.linkname = contents
                output.addfile(info)
            else:
                info.size = len(contents)
                output.addfile(info, io.BytesIO(contents))


def test_internal_interpreter_alias_is_materialized_before_publication(
    tmp_path: Path,
) -> None:
    source = tmp_path / "input.tar"
    archive(
        source,
        [
            ("python/bin/python3.11", b"interpreter"),
            ("python/bin/python3", "python3.11"),
        ],
    )
    root = extract_python(source, tmp_path / "unpacked")
    assert (root / "bin/python3").read_bytes() == b"interpreter"
    assert not (root / "bin/python3").is_symlink()
    archive_path = tmp_path / "runtime.tar.gz"
    archive_runtime(root, archive_path)
    with tarfile.open(archive_path) as output:
        assert all(item.isfile() or item.isdir() for item in output.getmembers())


@pytest.mark.parametrize("name", ["../escaped", "/python/absolute", "python/../escaped", "python\\escaped"])
def test_interpreter_archive_rejects_escape_paths(tmp_path: Path, name: str) -> None:
    source = tmp_path / "input.tar"
    archive(source, [(name, b"escape")])
    with pytest.raises(ValueError):
        extract_python(source, tmp_path / "unpacked")
    assert not (tmp_path / "escaped").exists()


def test_interpreter_archive_rejects_external_alias(tmp_path: Path) -> None:
    source = tmp_path / "input.tar"
    archive(source, [("python/bin/python3", "../../../external")])
    with pytest.raises((ValueError, tarfile.FilterError)):
        extract_python(source, tmp_path / "unpacked")


def test_runtime_archive_never_conveys_symbolic_links(tmp_path: Path) -> None:
    root = tmp_path / "runtime"
    root.mkdir()
    (root / "reference").write_bytes(b"native")
    (root / "alias").symlink_to("reference")
    with pytest.raises(ValueError, match="nonregular"):
        archive_runtime(root, tmp_path / "runtime.tar.gz")


def test_windows_native_extensions_are_signed_and_verified(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
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
    (root / "native").write_bytes(b"binary")
    os.link(root / "native", root / "alias")
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


def test_mac_signature_requirement_binds_approved_identity_and_team(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    (tmp_path / "python").write_bytes(b"\xcf\xfa\xed\xfe")
    identity = "Developer ID Application: Example Company (ABCDEFGHIJ)"
    monkeypatch.setenv("APPLE_SIGNING_IDENTITY", identity)
    commands = Mock()
    monkeypatch.setattr("runtime.signing.subprocess.run", commands)
    verify_runtime(tmp_path, "macos-arm64")
    command = commands.call_args.args[0]
    assert command[:4] == ["codesign", "--verify", "--strict", "-R"]
    assert 'certificate leaf[subject.CN] = "' + identity + '"' in command[4]
    assert 'certificate leaf[subject.OU] = "ABCDEFGHIJ"' in command[4]
