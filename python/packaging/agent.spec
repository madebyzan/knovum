# PyInstaller's standard onedir graph owns collection, relocation and bootloader signing.
import os
import sys
import hashlib
import json
from pathlib import Path
from PyInstaller.utils.hooks import collect_data_files, collect_submodules, copy_metadata
from PyInstaller.config import CONF

sys.path.insert(0, SPECPATH)
from policy import executable_options, runtime_distributions

root = Path(SPECPATH).parent
packages = [
    "knovum_agent_runtime",
    "deepagents",
    "langgraph",
    "langgraph.checkpoint.sqlite",
    "langchain_openai",
    "langchain_anthropic",
    "langchain_google_genai",
    "langchain_ollama",
    "langchain_deepseek",
    "langchain_groq",
    "truststore",
]
from importlib.metadata import distribution
import sysconfig

cpython_license = (
    Path(sys.base_prefix) if sys.platform == "win32" else Path(sysconfig.get_path("stdlib"))
) / "LICENSE.txt"
data = [
    (str(cpython_license), "licenses/CPython"),
    (
        str(
            distribution("pyinstaller").locate_file(
                f"pyinstaller-{distribution('pyinstaller').version}.dist-info/licenses/COPYING.txt"
            )
        ),
        "licenses/PyInstaller",
    ),
    (str(root / "uv.lock"), "build-evidence"),
    (str(root / "pyproject.toml"), "build-evidence"),
]
for name in ["agent.spec", "entry.py", "policy.py", "windows.manifest", "license-review.json"]:
    data.append((str(root / "packaging" / name), "build-evidence"))
hidden = []
for package in packages:
    hidden.extend(collect_submodules(package, filter=lambda name: ".tests" not in name))
    data.extend(collect_data_files(package, excludes=["**/tests/**"]))
for name in sorted(runtime_distributions()):
    data.extend(copy_metadata(name))

analysis = Analysis(
    [str(root / "packaging/entry.py")],
    pathex=[str(root / "src")],
    binaries=[],
    datas=data,
    hiddenimports=hidden,
    excludes=["pytest", "mypy", "ruff", "PyInstaller", "setuptools"],
    noarchive=False,
)
native_inputs = Path(CONF["workpath"]) / "agent-native-inputs.json"
native_inputs.write_text(
    json.dumps(
        {
            "schemaVersion": 1,
            "files": {
                name: hashlib.sha256(Path(source).read_bytes()).hexdigest()
                for name, source, kind in analysis.binaries
                if kind in {"BINARY", "EXTENSION"}
            },
        },
        sort_keys=True,
    )
)
archive = PYZ(analysis.pure)
options = executable_options(sys.platform)
if sys.platform == "darwin":
    options["codesign_identity"] = os.environ.get("APPLE_SIGNING_IDENTITY")
executable = EXE(
    archive,
    analysis.scripts,
    [("X utf8", None, "OPTION")],
    [],
    name="knovum-agent",
    debug=False,
    strip=False,
    upx=False,
    **options,
)
COLLECT(
    executable,
    analysis.binaries,
    analysis.datas,
    [("build-evidence/agent-native-inputs.json", str(native_inputs), "DATA")],
    strip=False,
    upx=False,
    name="knovum-agent",
)
