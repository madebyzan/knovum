import pytest

from knovum_inference.runtime.voice_device_probe import preferred_device


@pytest.mark.parametrize(("mask", "expected"), [(1, "cpu"), (3, "mps"), (5, "cuda"), (9, "vulkan"), (15, "cuda")])
def test_available_compiled_gpu_precedes_cpu(mask: int, expected: str) -> None:
    assert preferred_device(mask) == expected


@pytest.mark.parametrize("mask", [-1, 0, 2])
def test_failed_compiled_inventory_is_not_treated_as_cpu(mask: int) -> None:
    with pytest.raises(RuntimeError):
        preferred_device(mask)
