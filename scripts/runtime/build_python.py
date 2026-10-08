"""Produce one locked, relocatable Python runtime; no user-side installation."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
import tempfile
from pathlib import Path
from urllib.parse import urlparse

# Standalone and isolated execution authorize only this repository-owned tooling package.
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from runtime.artifacts import archive_runtime, digest
from runtime.commands import run
from runtime.conda_packages import dependency_digest, install_conda_packages
from runtime.dependencies import install_dependencies
from runtime.host import host_target
from runtime.signing import sign_runtime
from runtime.windows_process import configure_utf8

REPOSITORY = Path(__file__).resolve().parents[2]
PROJECT = REPOSITORY / "python"
ISOLATED_PYTHON_ARGS = ("-I", "-B", "-X", "utf8")


def relocated_smoke(runtime: Path, target: str, version: str, profile: str) -> None:
    executable = runtime / ("python.exe" if target == "windows-x64" else "bin/python3")
    imports = {
        "resources": "import huggingface_hub,modelscope_hub,pydantic,tqdm;assert 'torch' not in sys.modules",
        "light-inference": "sys.modules['torch']=None;import onnxruntime,sherpa_onnx,sentencepiece,soxr;"
        "from moss_tts_nano_ort.ort_cpu_runtime import OrtCpuRuntime;"
        + ("import faster_whisper" if target != "macos-arm64" else "import numpy"),
        "torch-inference": "import torch,sentence_transformers;"
        + ("import mlx_whisper" if target == "macos-arm64" else "import transformers")
        + (";import faster_whisper,ctranslate2;assert torch.version.cuda=='12.8'" if target == "windows-x64" else ""),
        "voice-generation": "sys.modules['torch']=None;import numpy,pydantic,ctypes;"
        "os.chdir(os.path.join(sys.prefix,'bin'));os.environ.pop('GGML_BACKEND_PATH',None);"
        "_backend_dll=os.add_dll_directory(os.getcwd()) if os.name=='nt' else None;"
        "_voice=ctypes.CDLL(os.path.join(sys.prefix,'compiled-libraries',"
        "{'darwin':'knovum_voice.dylib','win32':'knovum_voice.dll'}[sys.platform]));"
        "assert _voice.knovum_voice_abi()==2;assert _voice.knovum_voice_devices()>0;"
        "_align=ctypes.CDLL(os.path.join(sys.prefix,'compiled-libraries',"
        "{'darwin':'audiocpp.dylib','win32':'audiocpp.dll'}[sys.platform]));"
        "assert _align.audiocpp_abi_version()==0x000200;"
        "_registry=ctypes.c_void_p();assert _align.audiocpp_registry_create(None,ctypes.byref(_registry))==0;"
        "_families=[ctypes.c_char_p() for _ in range(_align.audiocpp_registry_family_count(_registry))];"
        "assert all(_align.audiocpp_registry_family(_registry,ctypes.c_size_t(i),ctypes.byref(name))==0 "
        "for i,name in enumerate(_families));"
        "assert {b'voxcpm2',b'qwen3_forced_aligner'}.issubset({name.value for name in _families});"
        "_align.audiocpp_registry_free(_registry)",
        "agent": "from deepagents import create_deep_agent;"
        "from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver;"
        "import langchain_openai,langchain_anthropic,langchain_google_genai;"
        "import langchain_ollama,langchain_deepseek,langchain_groq,truststore",
    }[profile]
    codepage = "import ctypes;assert ctypes.windll.kernel32.GetACP()==65001;" if target == "windows-x64" else ""
    code = (
        "import os,sys;_windows_dll_directory=os.add_dll_directory(os.path.join(sys.prefix,'Library','bin')) "
        "if os.name=='nt' else None;"
        + codepage
        + "sys.path.insert(0,sys.argv[1]);assert '.'.join(map(str,sys.version_info[:3]))==sys.argv[2];"
        + imports
    )
    environment = dict(
        os.environ,
        PYTHONDONTWRITEBYTECODE="1",
        HF_HUB_OFFLINE="1",
        HF_HUB_DISABLE_TELEMETRY="1",
        TRANSFORMERS_OFFLINE="1",
        PYANNOTE_METRICS_ENABLED="0",
        PYTORCH_ENABLE_MPS_FALLBACK="0",
    )
    for name in (
        "PYTHONPATH",
        "PYTHONHOME",
        "PYTHONSTARTUP",
        "HF_TOKEN",
        "HUGGING_FACE_HUB_TOKEN",
    ):
        environment.pop(name, None)
    run(
        str(executable),
        *ISOLATED_PYTHON_ARGS,
        "-c",
        code,
        str(runtime / "packages"),
        version,
        cwd=runtime.parent,
        env=environment,
    )


def build(
    output: Path,
    signing: str,
    archive_url: str | None,
    profile: str,
    scratch_parent: Path | None = None,
) -> None:
    if archive_url is not None:
        url = urlparse(archive_url)
        if url.scheme != "https" or not url.hostname or url.username or url.password or url.fragment:
            raise ValueError("Runtime publication requires one immutable HTTPS archive URL")
    if output.exists():
        raise ValueError("Runtime output already exists; use a fresh destination")
    target = host_target()
    config = json.loads((PROJECT / "runtime-source.json").read_text())
    actual_uv = run("uv", "--version", capture=True).split()[1]
    if actual_uv != config["uvVersion"]:
        raise ValueError(f"Runtime builder requires uv {config['uvVersion']}")
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="knovum-runtime-", dir=scratch_parent or output.parent) as directory:
        scratch = Path(directory)
        runtime, conda_inventory = install_conda_packages(PROJECT, profile, target, scratch)
        executable = runtime / ("python.exe" if target == "windows-x64" else "bin/python3")
        run(
            str(executable),
            *ISOLATED_PYTHON_ARGS,
            "-c",
            "import sys;assert '.'.join(map(str,sys.version_info[:3]))==sys.argv[1]",
            config["version"],
        )
        dependencies = install_dependencies(runtime, scratch, executable, REPOSITORY, profile, target)
        if target == "windows-x64":
            configure_utf8(executable, scratch / "build-evidence")
        relocated = scratch / "relocated runtime 中文"
        runtime.rename(relocated)
        signature = sign_runtime(relocated, target, signing)
        relocated_smoke(relocated, target, config["version"], profile)
        record = {
            "schemaVersion": 1,
            "profile": profile,
            "target": target,
            "version": config["version"],
            "condaSupplier": conda_inventory,
            "uvVersion": actual_uv,
            "dependencyLockSha256": dependency_digest(PROJECT, profile, target),
            "appCommit": run("git", "rev-parse", "HEAD", cwd=REPOSITORY, capture=True),
            "relocationVerified": True,
            "signing": signature,
            "licenseReview": "required",
        }
        if profile == "voice-generation":
            record["voxcpm"] = dependencies
        (scratch / "build-evidence/conda-packages.json").write_text(json.dumps(conda_inventory, indent=2) + "\n")
        (scratch / "build-evidence/producer.json").write_text(json.dumps(record, indent=2) + "\n")
        run(
            "node",
            str(REPOSITORY / "scripts/runtime/write-manifest.mjs"),
            "--target",
            target,
            "--directory",
            str(relocated),
            "--profile",
            profile,
        )
        publication = scratch / "publication"
        publication.mkdir()
        if archive_url is None:
            relocated.rename(publication / "runtime")
            shutil.copytree(scratch / "build-evidence", publication / "build-evidence")
            publication.rename(output)
            try:
                relocated_smoke(output / "runtime", target, config["version"], profile)
            except BaseException:
                shutil.rmtree(output)
                raise
            return
        artifact = publication / f"python-{profile}-{target}.tar.gz"
        archive_runtime(relocated, artifact)
        (publication / f"{artifact.name}.sha256").write_text(digest(artifact) + "\n")
        shutil.copytree(scratch / "build-evidence", publication / "build-evidence")
        evidence_archive = publication / f"python-{profile}-{target}-evidence.tar.gz"
        archive_runtime(publication / "build-evidence", evidence_archive)
        (publication / f"{evidence_archive.name}.sha256").write_text(digest(evidence_archive) + "\n")
        manifest = json.loads((relocated / "runtime-manifest.json").read_text())
        descriptor = {
            "schemaVersion": 1,
            "profile": profile,
            "target": target,
            "version": config["version"],
            "dependencyRequirementsSha256": manifest["dependencyRequirementsSha256"],
            "dependencyLockSha256": manifest["dependencyLockSha256"],
            "archiveUrl": archive_url,
            "size": artifact.stat().st_size,
            "sha256": digest(artifact),
            "manifestSha256": digest(relocated / "runtime-manifest.json"),
        }
        (publication / "runtime-descriptor.json").write_text(json.dumps(descriptor, indent=2) + "\n")
        publication.rename(output)


def arguments(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--profile",
        choices=(
            "resources",
            "light-inference",
            "torch-inference",
            "voice-generation",
            "agent",
        ),
        required=True,
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--scratch-parent",
        type=Path,
        help="Parent owned by the calling application's process scope",
    )
    parser.add_argument("--signing", choices=("adhoc", "unsigned", "production"))
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--archive-url", help="Immutable HTTPS publication location of the archive")
    mode.add_argument(
        "--development",
        action="store_true",
        help="Prepare a local runtime without publishing an archive",
    )
    args = parser.parse_args(argv)
    if args.archive_url and not args.signing:
        parser.error("publication requires --signing")
    if args.development and args.signing:
        parser.error("development selects host-local signing; omit --signing")
    return args


if __name__ == "__main__":
    args = arguments()
    signing = args.signing or ("adhoc" if host_target().startswith("macos-") else "unsigned")
    build(
        args.output.resolve(),
        signing,
        args.archive_url,
        args.profile,
        args.scratch_parent,
    )
