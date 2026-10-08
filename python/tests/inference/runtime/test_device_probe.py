from __future__ import annotations

import sys
import types
from unittest.mock import Mock

import pytest

from knovum_inference.runtime import device_probe

GIB = 1024**3


def backend(monkeypatch: pytest.MonkeyPatch, system: str, machine: str, cuda: bool, mps: bool, memory: int) -> Mock:
    properties = Mock(return_value=(memory, memory))
    monkeypatch.setattr(device_probe.platform, "system", lambda: system)
    monkeypatch.setattr(device_probe.platform, "machine", lambda: machine)
    monkeypatch.setitem(
        sys.modules,
        "torch",
        types.SimpleNamespace(
            cuda=types.SimpleNamespace(is_available=lambda: cuda, mem_get_info=properties),
            backends=types.SimpleNamespace(mps=types.SimpleNamespace(is_available=lambda: mps)),
            mps=types.SimpleNamespace(recommended_max_memory=properties),
        ),
    )
    return properties


def test_enumerated_intel_amd_gpu_does_not_qualify_torch_mps(monkeypatch: pytest.MonkeyPatch) -> None:
    properties = backend(monkeypatch, "Darwin", "x86_64", False, True, 4 * GIB)
    assert device_probe.torch_backends(4 * GIB) == {"cuda": False, "mps": False}
    properties.assert_not_called()


@pytest.mark.parametrize("memory,qualified", [(3 * GIB, False), (4 * GIB, True)])
def test_cuda_requires_the_frozen_memory_capacity(
    monkeypatch: pytest.MonkeyPatch, memory: int, qualified: bool
) -> None:
    properties = backend(monkeypatch, "Windows", "AMD64", True, False, memory)
    assert device_probe.torch_backends(4 * GIB) == {"cuda": qualified, "mps": False}
    properties.assert_called_once_with(0)


@pytest.mark.parametrize("memory,qualified", [(3 * GIB, False), (5 * GIB, True)])
def test_apple_silicon_mps_requires_the_frozen_memory_capacity(
    monkeypatch: pytest.MonkeyPatch, memory: int, qualified: bool
) -> None:
    properties = backend(monkeypatch, "Darwin", "arm64", False, True, memory)
    properties.return_value = memory
    assert device_probe.torch_backends(4 * GIB) == {"cuda": False, "mps": qualified}
    properties.assert_called_once_with()


def test_backend_detection_failure_is_an_error(monkeypatch: pytest.MonkeyPatch) -> None:
    properties = backend(monkeypatch, "Darwin", "arm64", False, True, 4 * GIB)
    properties.side_effect = RuntimeError("Metal initialization failed")
    with pytest.raises(RuntimeError, match="Metal initialization failed"):
        device_probe.torch_backends(4 * GIB)


@pytest.mark.parametrize("supported,qualified", [({"float16", "int8_float16"}, True), ({"int8_float32"}, False)])
def test_asr_cuda_requires_supported_float16_compute(
    monkeypatch: pytest.MonkeyPatch, supported: set[str], qualified: bool
) -> None:
    backend(monkeypatch, "Windows", "AMD64", True, False, 8 * GIB)
    compute_types = Mock(return_value=supported)
    monkeypatch.setitem(sys.modules, "ctranslate2", types.SimpleNamespace(get_supported_compute_types=compute_types))
    assert device_probe.transcription_backends(7 * GIB) == {"cuda": qualified, "mps": False}
    compute_types.assert_called_once_with("cuda", device_index=0)


def test_asr_does_not_initialize_cuda_on_a_cpu_computer(monkeypatch: pytest.MonkeyPatch) -> None:
    backend(monkeypatch, "Windows", "AMD64", False, False, 0)
    monkeypatch.setitem(sys.modules, "ctranslate2", None)
    assert device_probe.transcription_backends(2 * GIB) == {"cuda": False, "mps": False}


def test_cuda_qualification_uses_current_free_memory_and_recovers_when_another_application_releases_it(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    backend(monkeypatch, "Windows", "AMD64", True, False, 12 * GIB)
    free_memory = Mock(side_effect=[(2 * GIB, 12 * GIB), (8 * GIB, 12 * GIB)])
    torch = sys.modules["torch"]
    monkeypatch.setattr(torch.cuda, "mem_get_info", free_memory, raising=False)
    assert device_probe.torch_backends(6 * GIB) == {"cuda": False, "mps": False}
    assert device_probe.torch_backends(6 * GIB) == {"cuda": True, "mps": False}
    assert free_memory.call_count == 2
