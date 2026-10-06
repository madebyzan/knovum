from __future__ import annotations

import os
import sqlite3
import subprocess
import sys
from pathlib import Path

import pytest


@pytest.mark.parametrize("crash", ["before_write", "after_write"])
def test_process_crash_preserves_tool_identity_and_completed_model_node(tmp_path: Path, crash: str) -> None:
    tests = Path(__file__).resolve().parents[1]
    environment = {**os.environ, "PYTHONPATH": str(tests)}
    command = [sys.executable, "-m", "agent_runtime.sdk_crash_probe", str(tmp_path)]
    stopped = subprocess.run([*command, crash], env=environment, capture_output=True, timeout=60)
    assert stopped.returncode == 86, stopped.stderr.decode()
    resumed = subprocess.run([*command, "resume"], env=environment, capture_output=True, timeout=60)
    assert resumed.returncode == 0, resumed.stderr.decode()
    with sqlite3.connect(tmp_path / "host.sqlite") as database:
        assert database.execute("SELECT COUNT(*) FROM operations").fetchone()[0] == 1
