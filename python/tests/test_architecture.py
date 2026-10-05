from __future__ import annotations

import ast
import subprocess
import sys
from pathlib import Path

import pytest

SOURCE = Path(__file__).resolve().parents[1] / "src"
PACKAGE = SOURCE / "knovum_inference"


@pytest.mark.parametrize("owner", ["contracts", "transport", "speech", "embedding.py", "image_segmentation.py"])
def test_implementation_owners_do_not_depend_on_runtime_composition_or_tools(owner: str) -> None:
    path = PACKAGE / owner
    modules = [path] if path.is_file() else list(path.rglob("*.py"))
    allowed = {"errors", owner.split(".")[0], "contracts"}
    for module in modules:
        for node in ast.walk(ast.parse(module.read_text())):
            imports = (
                [node.module or ""]
                if isinstance(node, ast.ImportFrom)
                else [alias.name for alias in node.names]
                if isinstance(node, ast.Import)
                else []
            )
            for imported in imports:
                assert not imported.startswith(("runtime_packaging", "acceptance")), str(module)
                if imported.startswith("knovum_inference."):
                    dependency = imported.split(".")[1]
                    assert dependency in allowed, f"{module}: forbidden dependency {imported}"


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
"""
    process = subprocess.run(
        [sys.executable, "-I", "-B", "-c", script, str(SOURCE)], capture_output=True, text=True, timeout=10, check=False
    )
    assert process.returncode == 0, process.stderr
