from __future__ import annotations

import subprocess
from pathlib import Path


def run(*arguments: str, cwd: Path | None = None, capture: bool = False, env: dict[str, str] | None = None) -> str:
    command = subprocess.run(
        arguments, cwd=cwd, env=env, check=True, text=True, stdout=subprocess.PIPE if capture else None
    )
    return command.stdout.strip() if capture else ""
