import os
from pathlib import Path

import pytest

from distribution import seal, verify


def test_sealed_distribution_rejects_corruption_and_preserves_safe_links(tmp_path: Path):
    root = tmp_path / "知识空间"
    (root / "_internal").mkdir(parents=True)
    (root / "knovum-agent").write_bytes(b"executable")
    (root / "_internal/library").write_bytes(b"library")
    if os.name != "nt":
        (root / "library").symlink_to("_internal/library")
    trusted = seal(root, "macos-x64")
    verify(root, trusted)
    (root / "_internal/library").write_bytes(b"tampered")
    with pytest.raises(ValueError, match="differs"):
        verify(root, trusted)


@pytest.mark.parametrize("name", ["unexpected.txt", "_internal/agent-manifest.json"])
def test_sealed_distribution_rejects_every_unlisted_file(tmp_path: Path, name: str):
    root = tmp_path / "agent"
    (root / "_internal").mkdir(parents=True)
    (root / "knovum-agent").write_bytes(b"executable")
    trusted = seal(root, "macos-x64")
    (root / name).write_bytes(b"unlisted")
    with pytest.raises(ValueError, match="differs"):
        verify(root, trusted)


@pytest.mark.skipif(os.name == "nt", reason="POSIX onedir distributions own symlink preservation")
def test_link_outside_bundle_cannot_be_sealed(tmp_path: Path):
    root = tmp_path / "agent"
    root.mkdir()
    (tmp_path / "secret").write_bytes(b"external")
    (root / "knovum-agent").write_bytes(b"executable")
    (root / "escape").symlink_to("../secret")
    with pytest.raises(ValueError, match="escapes"):
        seal(root, "macos-x64")
