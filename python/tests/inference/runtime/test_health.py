from __future__ import annotations

import pytest

from knovum_inference.errors import RpcError
from knovum_inference.runtime import health


@pytest.mark.parametrize(
    ("system", "machine", "reader"),
    [
        ("Darwin", "arm64", "_macos_resident_bytes"),
        ("Darwin", "x86_64", "_macos_resident_bytes"),
        ("Windows", "AMD64", "_windows_resident_bytes"),
        ("Windows", "x86_64", "_windows_resident_bytes"),
    ],
)
def test_resident_memory_uses_the_supported_platform_reader(
    monkeypatch: pytest.MonkeyPatch,
    system: str,
    machine: str,
    reader: str,
) -> None:
    monkeypatch.setattr(health.platform, "system", lambda: system)
    monkeypatch.setattr(health.platform, "machine", lambda: machine)
    monkeypatch.setattr(health, reader, lambda: 42_000_000)

    assert health.current_process_resident_bytes() == 42_000_000


def test_resident_memory_rejects_unshipped_platforms(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(health.platform, "system", lambda: "Linux")
    monkeypatch.setattr(health.platform, "machine", lambda: "x86_64")

    with pytest.raises(RpcError) as captured:
        health.current_process_resident_bytes()

    assert captured.value.stable_code == "unsupported_device"


@pytest.mark.parametrize("resident_bytes", [0, -1])
def test_resident_memory_rejects_nonpositive_os_results(
    monkeypatch: pytest.MonkeyPatch,
    resident_bytes: int,
) -> None:
    monkeypatch.setattr(health.platform, "system", lambda: "Darwin")
    monkeypatch.setattr(health.platform, "machine", lambda: "arm64")
    monkeypatch.setattr(health, "_macos_resident_bytes", lambda: resident_bytes)

    with pytest.raises(RpcError) as captured:
        health.current_process_resident_bytes()

    assert captured.value.stable_code == "runtime_mismatch"
