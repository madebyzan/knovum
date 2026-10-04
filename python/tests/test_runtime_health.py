from __future__ import annotations

import pytest

from knovum_inference import runtime_health
from knovum_inference.errors import RpcError


@pytest.mark.parametrize(
    ("system", "machine", "reader"),
    [
        ("Darwin", "arm64", "_macos_resident_bytes"),
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
    monkeypatch.setattr(runtime_health.platform, "system", lambda: system)
    monkeypatch.setattr(runtime_health.platform, "machine", lambda: machine)
    monkeypatch.setattr(runtime_health, reader, lambda: 42_000_000)

    assert runtime_health.current_process_resident_bytes() == 42_000_000


def test_resident_memory_rejects_unshipped_platforms(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(runtime_health.platform, "system", lambda: "Linux")
    monkeypatch.setattr(runtime_health.platform, "machine", lambda: "x86_64")

    with pytest.raises(RpcError) as captured:
        runtime_health.current_process_resident_bytes()

    assert captured.value.stable_code == "unsupported_device"


@pytest.mark.parametrize("resident_bytes", [0, -1])
def test_resident_memory_rejects_nonpositive_os_results(
    monkeypatch: pytest.MonkeyPatch,
    resident_bytes: int,
) -> None:
    monkeypatch.setattr(runtime_health.platform, "system", lambda: "Darwin")
    monkeypatch.setattr(runtime_health.platform, "machine", lambda: "arm64")
    monkeypatch.setattr(runtime_health, "_macos_resident_bytes", lambda: resident_bytes)

    with pytest.raises(RpcError) as captured:
        runtime_health.current_process_resident_bytes()

    assert captured.value.stable_code == "runtime_mismatch"
