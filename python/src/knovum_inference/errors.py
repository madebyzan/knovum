from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(slots=True)
class RpcError(Exception):
    code: int
    message: str
    stable_code: str
    details: dict[str, Any] | None = None

    def as_object(self) -> dict[str, Any]:
        data: dict[str, Any] = {"code": self.stable_code}
        if self.details:
            data.update(self.details)
        return {"code": self.code, "message": self.message, "data": data}


def invalid_params(message: str) -> RpcError:
    return RpcError(-32602, message, "invalid_output")


def server_error(stable_code: str, message: str) -> RpcError:
    return RpcError(-32000, message, stable_code)
