"""Produce one locked, relocatable native Python runtime; no user-side installation."""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import sys
import tempfile
from pathlib import Path
from urllib.parse import urlparse

# Standalone and isolated execution authorize only this repository-owned tooling package.
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from runtime.artifacts import archive_runtime, digest, download, extract_python
from runtime.commands import run
from runtime.dependencies import install_dependencies
from runtime.signing import sign_runtime

REPOSITORY = Path(__file__).resolve().parents[2]
PROJECT = REPOSITORY / "python"
ISOLATED_PYTHON_ARGS = ("-I", "-B", "-X", "utf8")


def host_target() -> str:
    machine = platform.machine().lower()
    if platform.system() == "Darwin" and machine == "arm64":
        return "macos-arm64"
    if platform.system() == "Darwin" and machine == "x86_64":
        return "macos-x64"
    if platform.system() == "Windows" and machine in {"amd64", "x86_64"}:
        return "windows-x64"
    raise ValueError(f"Unsupported native runtime builder: {platform.system()}/{machine}")


def relocated_smoke(runtime: Path, target: str, version: str, profile: str) -> None:
    executable = runtime / ("python.exe" if target == "windows-x64" else "bin/python3")
    imports = {
        "resources": "import huggingface_hub,modelscope_hub,pydantic,tqdm;assert 'torch' not in sys.modules",
        "light-inference": "sys.modules['torch']=None;import onnxruntime,sherpa_onnx;"
        + ("import faster_whisper" if target != "macos-arm64" else "import numpy"),
        "torch-inference": "import torch,sentence_transformers;from pyannote.audio import Pipeline;"
        + ("import mlx_whisper" if target == "macos-arm64" else "import transformers"),
        "voice-generation": "import torch,voxcpm,silero_vad",
        "agent": "from deepagents import create_deep_agent;"
        "from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver;"
        "import langchain_openai,langchain_anthropic,langchain_google_genai;"
        "import langchain_ollama,langchain_deepseek,langchain_groq,truststore",
    }[profile]
    code = (
        "import sys;sys.path.insert(0,sys.argv[1]);assert '.'.join(map(str,sys.version_info[:3]))==sys.argv[2];"
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
    for name in ("PYTHONPATH", "PYTHONHOME", "PYTHONSTARTUP", "HF_TOKEN", "HUGGING_FACE_HUB_TOKEN"):
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


def build(output: Path, signing: str, archive_url: str, profile: str) -> None:
    url = urlparse(archive_url)
    if url.scheme != "https" or not url.hostname or url.username or url.password or url.fragment:
        raise ValueError("Runtime publication requires one immutable HTTPS archive URL")
    target = host_target()
    config = json.loads((PROJECT / "runtime-source.json").read_text())
    actual_uv = run("uv", "--version", capture=True).split()[1]
    if actual_uv != config["uvVersion"]:
        raise ValueError(f"Runtime builder requires uv {config['uvVersion']}")
    if output.exists():
        raise ValueError("Runtime output already exists; use a fresh destination")
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="knovum-runtime-", dir=output.parent) as directory:
        scratch = Path(directory)
        archive = scratch / "python.tar.gz"
        source = config["targets"][target]
        download(source, archive)
        runtime = extract_python(archive, scratch)
        executable = runtime / ("python.exe" if target == "windows-x64" else "bin/python3")
        run(
            str(executable),
            *ISOLATED_PYTHON_ARGS,
            "-c",
            "import sys;assert '.'.join(map(str,sys.version_info[:3]))==sys.argv[1]",
            config["version"],
        )
        dependencies = install_dependencies(runtime, scratch, executable, REPOSITORY, profile, target)
        relocated = scratch / "relocated runtime 中文"
        runtime.rename(relocated)
        relocated_smoke(relocated, target, config["version"], profile)
        signature = sign_runtime(relocated, target, signing)
        relocated_smoke(relocated, target, config["version"], profile)
        record = {
            "schemaVersion": 1,
            "profile": profile,
            "target": target,
            "version": config["version"],
            "standaloneRelease": config["standaloneRelease"],
            "interpreterSource": source,
            "uvVersion": actual_uv,
            "dependencyLockSha256": digest((PROJECT / "runtimes" / profile) / "uv.lock"),
            "appCommit": run("git", "rev-parse", "HEAD", cwd=REPOSITORY, capture=True),
            "relocationVerified": True,
            "signing": signature,
            "licenseReview": "required",
        }
        if profile == "voice-generation":
            record["voxcpm"] = dependencies
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


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--profile",
        choices=("resources", "light-inference", "torch-inference", "voice-generation", "agent"),
        required=True,
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--signing", choices=("adhoc", "unsigned", "production"), required=True)
    parser.add_argument("--archive-url", required=True, help="Immutable HTTPS publication location of the archive")
    args = parser.parse_args()
    build(args.output.resolve(), args.signing, args.archive_url, args.profile)
