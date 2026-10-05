from __future__ import annotations

import hashlib
from pathlib import Path

import pytest

from knovum_inference.errors import RpcError
from knovum_inference.runtime import identity


def test_runtime_manifest_path_is_independent_of_worker_location(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    module = tmp_path / "runtime 中文/packages/knovum_inference/runtime/identity.py"
    module.parent.mkdir(parents=True)
    manifest = tmp_path / "separate runtime/runtime-manifest.json"
    manifest.parent.mkdir()
    manifest.write_bytes(b'{"runtime":"verified"}')
    monkeypatch.setattr(identity, "__file__", str(module))

    identity.verify_runtime_digest(hashlib.sha256(manifest.read_bytes()).hexdigest(), manifest)

    with pytest.raises(RpcError) as caught:
        identity.verify_runtime_digest("0" * 64, manifest)
    assert caught.value.stable_code == "runtime_mismatch"
    manifest.unlink()
    with pytest.raises(RpcError) as caught:
        identity.verify_runtime_digest("0" * 64, manifest)
    assert caught.value.stable_code == "runtime_missing"
