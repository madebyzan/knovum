from __future__ import annotations

import ast
import importlib.util
import subprocess
import sys
from pathlib import Path

import pytest

SOURCE = Path(__file__).resolve().parents[1] / "src"
DEPENDENCIES = {
    "knovum_inference": {
        "__init__": set(),
        "__main__": {"server"},
        "server": {"runtime"},
        "runtime": {"contracts", "embedding", "errors", "image_segmentation", "speech", "transport"},
        "transport": {"contracts", "errors"},
        "contracts": set(),
        "errors": set(),
        "speech": {"contracts", "errors"},
        "embedding": {"errors"},
        "image_segmentation": {"errors"},
    },
}
DEPENDENCIES["knovum_model_resources"] = {
    "__init__": set(),
    "wire_model": set(),
    "contracts": {"wire_model"},
    "progress": set(),
    "hugging_face": {"contracts", "progress"},
    "model_scope": {"contracts", "progress"},
    "server": {"contracts", "progress", "hugging_face", "model_scope"},
}
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
        if len(parts) > 1 and parts[1] in DEPENDENCIES[root]:
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
        "import knovum_agent_runtime",
        "from scripts.runtime import artifacts",
        "from acceptance import evaluator",
    ],
)
def test_capability_boundary_rejects_absolute_relative_and_package_imports(source: str) -> None:
    with pytest.raises(AssertionError):
        check_dependencies(source, "knovum_inference.speech.alignment", "speech")


@pytest.mark.parametrize(
    "source",
    ["from ..errors import server_error", "from .request import verify_audio", "from ..contracts import wire_model"],
)
def test_capability_boundary_accepts_owned_dependencies(source: str) -> None:
    check_dependencies(source, "knovum_inference.speech.alignment", "speech")


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
import knovum_inference.server
import knovum_model_resources.server
"""
    process = subprocess.run(
        [sys.executable, "-I", "-B", "-c", script, str(SOURCE)], capture_output=True, text=True, timeout=10, check=False
    )
    assert process.returncode == 0, process.stderr
