from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
import threading
from pathlib import Path
from queue import Queue

from knovum_inference import PROTOCOL_VERSION


def test_closed_transport_terminates_stuck_native_initialization() -> None:
    source = Path(__file__).resolve().parents[3] / "src"
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
            "runtimeManifestPath": "unused",
            "runtimeDigest": "digest",
            "sessionId": "session",
            "capability": "embedding",
            "capabilitySpecId": "spec",
            "deviceProfile": {"device": "cpu", "executionProfileId": "cpu", "cpuThreads": 4},
            "workDirectory": "unused",
            "modelPaths": {"embedding": "unused"},
        },
    }
    process = subprocess.Popen(
        [sys.executable, "-c", script],
        env={**os.environ, "PYTHONPATH": str(source)},
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=os.name == "posix",
    )
    assert process.stdin is not None and process.stderr is not None
    started: Queue[bytes] = Queue()
    diagnostics = process.stderr
    reader = threading.Thread(target=lambda: started.put(diagnostics.readline()), daemon=True)
    reader.start()
    try:
        process.stdin.write(json.dumps(request).encode() + b"\n")
        process.stdin.flush()
        # Startup/import time is separate from the five-second shutdown contract.
        assert started.get(timeout=30) == b"native-initialization-started\n"
        reader.join(timeout=1)
        assert not reader.is_alive()
        process.stdin.close()
        process.stdin = None
        stdout, _stderr = process.communicate(timeout=10)
        assert process.returncode == (-signal.SIGKILL if os.name == "posix" else 1)
        assert stdout == b""
    finally:
        if process.poll() is None:
            process.kill()
        process.communicate(timeout=10)
        reader.join(timeout=1)
