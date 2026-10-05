from __future__ import annotations

import json
from typing import Any, TypeVar

from pydantic import BaseModel, ValidationError

from knovum_inference.errors import RpcError, invalid_params, server_error

T = TypeVar("T", bound=BaseModel)
_OOM_MESSAGES = (
    "out of memory",
    "not enough memory",
    "failed to allocate memory",
    "cannot allocate memory",
    "can't allocate memory",
    "memory allocation failed",
    "std::bad_alloc",
)


def validate_params(model: type[T], value: Any) -> T:
    try:
        return _decode_wire_model(model, value)
    except (TypeError, ValueError, ValidationError) as error:
        raise invalid_params("Parameters do not match the runtime contract") from error


def validate_result(model: type[T], value: dict[str, Any]) -> dict[str, Any]:
    try:
        return _decode_wire_model(model, value).model_dump(by_alias=True, mode="json")
    except (TypeError, ValueError, ValidationError) as error:
        raise server_error("invalid_output", "Worker result violates its contract") from error


def _decode_wire_model(model: type[T], value: Any) -> T:
    encoded = json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(",", ":"))
    return model.model_validate_json(encoded)


def _out_of_memory(error: BaseException) -> bool:
    current: BaseException | None = error
    visited: set[int] = set()
    while current is not None and id(current) not in visited:
        visited.add(id(current))
        if isinstance(current, MemoryError):
            return True
        error_type = type(current)
        if error_type.__name__ == "OutOfMemoryError" and error_type.__module__.startswith("torch"):
            return True
        message = str(current).lower()
        if any(marker in message for marker in _OOM_MESSAGES):
            return True
        current = current.__cause__ or current.__context__
    return False


def report_request_error(error: Exception) -> RpcError:
    if _out_of_memory(error):
        return server_error("resource_exhausted", "Inference exhausted its memory budget")
    if isinstance(error, RpcError):
        return error
    if isinstance(error, ImportError):
        return server_error("runtime_mismatch", "Bundled inference dependency versions do not match")
    return server_error("invalid_output", "Worker request failed")
