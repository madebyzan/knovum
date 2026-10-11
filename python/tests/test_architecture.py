from __future__ import annotations

import ast
import importlib.util
import subprocess
import sys
from pathlib import Path

import pytest

SOURCE = Path(__file__).resolve().parents[1] / "src"
DEPENDENCIES = {}
DEPENDENCIES["knovum_agent_runtime"] = {
    "__init__": set(),
    "__main__": {"server"},
    "contracts": set(),
    "transport": set(),
    "server": {"contracts", "execution", "projection", "retention", "transport"},
    "retention": {"contracts"},
    "execution": {"composition", "contracts", "control", "inputs", "projection", "transport"},
    "composition": {"contracts", "control", "host_tools", "model_binding", "skills"},
    "skills": {"contracts"},
    "control": {"contracts", "projection", "transport"},
    "inputs": {"contracts", "host_tools"},
    "host_tools": {"contracts", "control"},
    "budget": {"contracts", "control"},
    "model_binding": {"budget", "contracts"},
    "checkpoints": set(),
    "projection": {"checkpoints", "contracts", "transport"},
}
TOOLING_PACKAGES = {"scripts", "tests", "acceptance", "runtime", "generate_python_contracts"}


def imports(source: str, package: str) -> set[str]:
    dependencies: set[str] = set()
    for node in ast.walk(ast.parse(source)):
        if isinstance(node, ast.Import):
            dependencies.update(alias.name for alias in node.names)
        elif isinstance(node, ast.ImportFrom):
            module = node.module or ""
            if node.level:
                module = importlib.util.resolve_name("." * node.level + module, package)
            dependencies.add(module)
            dependencies.update(f"{module}.{alias.name}" for alias in node.names if alias.name != "*")
    return dependencies


def check_dependencies(source: str, module: str, owner: str) -> None:
    package = module.rsplit(".", 1)[0]
    root = module.split(".")[0]
    allowed = DEPENDENCIES[root][owner] | {owner}
    for dependency in imports(source, package):
        parts = dependency.split(".")
        assert parts[0] not in TOOLING_PACKAGES, f"{module}: tooling dependency {dependency}"
        if not parts[0].startswith("knovum_"):
            continue
        assert parts[0] == root, f"{module}: undeclared cross-capability dependency {dependency}"
        if len(parts) > 1:
            assert parts[1] in DEPENDENCIES[root], f"{module}: unknown dependency {dependency}"
            assert parts[1] in allowed, f"{module}: forbidden dependency {dependency}"


def test_all_source_packages_have_explicit_dependency_and_test_ownership() -> None:
    packages = {path.name for path in SOURCE.iterdir() if path.is_dir() and path.name != "__pycache__"}
    assert packages == DEPENDENCIES.keys()
    for package in packages:
        assert (SOURCE.parent / "tests" / package.removeprefix("knovum_")).is_dir()
        modules = list((SOURCE / package).rglob("*.py"))
        owners = {path.relative_to(SOURCE / package).parts[0].removesuffix(".py") for path in modules}
        assert owners == DEPENDENCIES[package].keys()
        for path in modules:
            module = ".".join(path.relative_to(SOURCE).with_suffix("").parts)
            owner = path.relative_to(SOURCE / package).parts[0].removesuffix(".py")
            check_dependencies(path.read_text(), module, owner)


@pytest.mark.parametrize(
    "source",
    [
        "import knovum_inference.runtime.worker",
        "from knovum_inference import runtime",
        "from ..runtime import worker",
        "from .. import runtime",
        "from scripts.runtime import artifacts",
        "from acceptance import evaluator",
    ],
)
def test_capability_boundary_rejects_absolute_relative_and_package_imports(source: str) -> None:
    with pytest.raises((AssertionError, ImportError)):
        check_dependencies(source, "knovum_agent_runtime.skills", "skills")


@pytest.mark.parametrize(
    "source",
    ["from .contracts import wire_model", "from .skills import Skill", "from . import contracts"],
)
def test_capability_boundary_accepts_owned_dependencies(source: str) -> None:
    check_dependencies(source, "knovum_agent_runtime.skills", "skills")


def test_worker_entry_import_does_not_load_unselected_inference_libraries() -> None:
    script = """
import importlib.abc
import sys
sys.path.insert(0, sys.argv[1])

class RejectEngineImports(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname.split('.')[0] in {
            'torch', 'onnxruntime', 'sentence_transformers', 'transformers',
            'sherpa_onnx', 'voxcpm', 'pyannote', 'mlx', 'mlx_whisper',
            'faster_whisper', 'silero_vad', 'numpy',
        }:
            raise AssertionError('Worker imported an unselected inference library: ' + fullname)

sys.meta_path.insert(0, RejectEngineImports())
import knovum_agent_runtime.transport
"""
    process = subprocess.run(
        [sys.executable, "-I", "-B", "-c", script, str(SOURCE)], capture_output=True, text=True, timeout=10, check=False
    )
    assert process.returncode == 0, process.stderr


def test_control_initialization_and_malformed_requests_do_not_load_model_providers(tmp_path: Path) -> None:
    script = """
import importlib.abc
import sys
class RejectProviders(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        providers = {'deepagents', 'langchain_openai', 'langchain_anthropic', 'langchain_google_genai',
            'langchain_ollama', 'langchain_deepseek', 'langchain_groq'}
        if fullname.startswith('langgraph.graph') or fullname.split('.')[0] in providers:
            raise AssertionError('Control initialization loaded a model provider: ' + fullname)
sys.meta_path.insert(0, RejectProviders())
import knovum_agent_runtime.server
import asyncio
import io
from contextlib import AsyncExitStack
from knovum_agent_runtime.server import Server, AgentInitialize, GRAPH_VERSION, PROTOCOL_VERSION
from knovum_agent_runtime.transport import Peer, HostRejected
async def run():
    async with AsyncExitStack() as lifetime:
        server = Server(Peer(io.BytesIO(), io.BytesIO()), lifetime)
        await server.initialize(AgentInitialize(protocol_version=PROTOCOL_VERSION,
            graph_version=GRAPH_VERSION, checkpoint_path=sys.argv[1]))
        try:
            await server.dispatch('agentRuntime.execute', {})
        except HostRejected as error:
            assert error.code == -32602
        else:
            raise AssertionError('Malformed execution request was accepted')
asyncio.run(run())
"""
    process = subprocess.run(
        [sys.executable, "-I", "-B", "-c", script, str(tmp_path / "checkpoint.sqlite")],
        capture_output=True,
        text=True,
        timeout=20,
    )
    assert process.returncode == 0, process.stderr
