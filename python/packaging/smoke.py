"""Qualify a relocated frozen Agent with the product's own duplex protocol."""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from distribution import MANIFEST, verify
from knovum_agent_runtime.contracts.generated.control import AgentExecute, AgentInitialize
from knovum_agent_runtime.contracts.values import GRAPH_VERSION, PROTOCOL_VERSION
from knovum_agent_runtime.transport import HostRejected, Peer


def execution_request() -> AgentExecute:
    return AgentExecute.model_validate_json(
        json.dumps(
            {
                "identity": {"runId": "smoke", "threadId": "smoke", "executionRevision": "1", "controlEpoch": 0},
                "model": {
                    "protocol": "open_ai_chat",
                    "model": "packaging-smoke",
                    "baseUrl": "https://invalid.example/v1",
                    "apiVersion": None,
                    "apiKey": "test-only",
                    "temperature": 0.0,
                    "maxOutputTokens": 100,
                    "maxInputTokens": 32768,
                },
                "systemPrompt": "Packaging qualification.",
                "tools": [],
                "messages": [{"id": "input:1", "role": "user", "content": [{"type": "text", "text": "hello"}]}],
                "inputId": "input:1",
                "afterCheckpointId": None,
                "resume": None,
                "skills": [],
                "skillToolBindings": {},
                "memory": {},
            }
        )
    )


async def qualify(process: subprocess.Popen[bytes], checkpoint: Path) -> None:
    assert process.stdin is not None and process.stdout is not None
    peer = Peer(process.stdout, process.stdin)
    recovered = False

    async def deny_execution(method: str, _value: Any) -> Any:
        nonlocal recovered
        assert method == "agentRuntime.requireRecovery", method
        recovered = True
        raise HostRejected(-32000, "packaging-smoke-denied")

    listener = asyncio.create_task(peer.run(deny_execution))
    try:
        ready = await asyncio.wait_for(
            peer.request(
                "agentRuntime.initialize",
                AgentInitialize(
                    protocol_version=PROTOCOL_VERSION,
                    graph_version=GRAPH_VERSION,
                    checkpoint_path=str(checkpoint),
                ),
            ),
            20,
        )
        assert ready == {"protocolVersion": PROTOCOL_VERSION, "graphVersion": GRAPH_VERSION}, ready
        assert checkpoint.is_file()
        # Graph construction has no product handshake deadline. Bound build qualification
        # separately, and deny the first host admission before any model/provider call.
        try:
            await asyncio.wait_for(peer.request("agentRuntime.execute", execution_request()), 60)
        except HostRejected as error:
            assert recovered and error.code == -32000 and str(error) == "packaging-smoke-denied", error
        else:
            raise AssertionError("Execution passed the denied host admission")
    finally:
        peer.close()
        await listener


def smoke(executable: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="knovum-agent-知识空间-") as temporary:
        relocated = Path(temporary) / "已安装 Agent"
        shutil.copytree(executable.resolve().parent, relocated, symlinks=True)
        verify(relocated, (executable.resolve().parent / MANIFEST).read_bytes())
        environment = {key: os.environ[key] for key in ("SYSTEMROOT", "WINDIR", "TEMP", "TMP") if key in os.environ}
        process = subprocess.Popen(
            [str(relocated / executable.name)],
            cwd=temporary,
            env=environment,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            creationflags=0x08000000 if os.name == "nt" else 0,
        )
        try:
            asyncio.run(qualify(process, Path(temporary) / "检查点.sqlite"))
            assert process.stdin is not None
            process.stdin.close()
            assert process.wait(timeout=10) == 0
        except BaseException:
            process.kill()
            process.wait(timeout=10)
            assert process.stderr is not None
            print(process.stderr.read().decode(errors="replace"))
            raise
        finally:
            for stream in (process.stdin, process.stdout, process.stderr):
                if stream is not None:
                    stream.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    smoke(parser.parse_args().executable)
