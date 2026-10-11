from __future__ import annotations

import os
import tarfile
from pathlib import Path

import pytest

from runtime.artifacts import archive_runtime, materialize_aliases


def test_runtime_archive_never_conveys_symbolic_links(tmp_path: Path) -> None:
    root = tmp_path / "runtime"
    root.mkdir()
    (root / "reference").write_bytes(b"compiled")
    (root / "alias").symlink_to("reference")
    with pytest.raises(ValueError, match="nonregular"):
        archive_runtime(root, tmp_path / "runtime.tar.gz")


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
