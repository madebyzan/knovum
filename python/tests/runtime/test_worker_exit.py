from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
from pathlib import Path

from knovum_inference import PROTOCOL_VERSION


def test_closed_transport_terminates_stuck_native_initialization() -> None:
    source = Path(__file__).resolve().parents[2] / "src"
    script = """
import asyncio
import sys
import time
from knovum_inference.runtime import worker

def prepare(params):
    print("native-initialization-started", file=sys.stderr, flush=True)
    time.sleep(60)

worker.prepare_session = prepare
asyncio.run(worker.Worker().serve())
"""
    request = {
        "jsonrpc": "2.0",
        "id": "init",
        "method": "runtime.initialize",
        "params": {
            "protocolVersion": PROTOCOL_VERSION,
            "runtimeDigest": "digest",
            "sessionId": "session",
            "capability": "embedding",
            "capabilitySpecId": "spec",
            "deviceProfile": {"device": "cpu", "executionProfileId": "cpu", "cpuThreads": 4},
            "workDirectory": "unused",
            "modelPaths": {"embedding": "unused"},
        },
    }
    process = subprocess.run(
        [sys.executable, "-c", script],
        env={**os.environ, "PYTHONPATH": str(source)},
        input=json.dumps(request).encode() + b"\n",
        capture_output=True,
        timeout=10,
        start_new_session=os.name == "posix",
        check=False,
    )
    assert b"native-initialization-started" in process.stderr
    assert process.returncode == (-signal.SIGKILL if os.name == "posix" else 1)
    assert process.stdout == b""
