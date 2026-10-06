"""Produce one locked, relocatable native Python runtime; no user-side installation."""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from urllib.parse import urlparse

# Standalone and isolated execution authorize only this repository-owned tooling package.
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from runtime.artifacts import archive_runtime, digest, download, extract_python
from runtime.inventory import write_inventory
from runtime.signing import sign_runtime

REPOSITORY = Path(__file__).resolve().parents[2]
PROJECT = REPOSITORY / "python"
ISOLATED_PYTHON_ARGS = ("-I", "-B", "-X", "utf8")


def run(*arguments: str, cwd: Path | None = None, capture: bool = False, env: dict[str, str] | None = None) -> str:
    command = subprocess.run(
        arguments, cwd=cwd, env=env, check=True, text=True, stdout=subprocess.PIPE if capture else None
    )
    return command.stdout.strip() if capture else ""


def host_target() -> str:
    machine = platform.machine().lower()
    if platform.system() == "Darwin" and machine == "arm64":
        return "macos-arm64"
    if platform.system() == "Windows" and machine in {"amd64", "x86_64"}:
        return "windows-x64"
    raise ValueError(f"Unsupported native runtime builder: {platform.system()}/{machine}")


def export_requirements(destination: Path, *options: str) -> None:
    run(
        "uv",
        "export",
        "--project",
        str(PROJECT),
        "--frozen",
        "--no-emit-project",
        "--no-header",
        "--no-annotate",
        "--no-default-groups",
        *options,
        "--output-file",
        str(destination),
    )


def install_dependencies(runtime: Path, scratch: Path, executable: Path) -> dict[str, str]:
    environment = dict(
        os.environ,
        UV_LINK_MODE="copy",
        UV_PYTHON_DOWNLOADS="never",
        PYTHONDONTWRITEBYTECODE="1",
        PYTORCH_ENABLE_MPS_FALLBACK="0",
    )
    build_environment = scratch / "build-environment"
    run("uv", "venv", "--python", str(executable), str(build_environment), env=environment)
    build_python = build_environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    build_requirements, requirements = scratch / "build-requirements.txt", scratch / "requirements.txt"
    export_requirements(build_requirements, "--only-group", "runtime-build")
    export_requirements(
        requirements, "--extra", "inference", "--extra", "model-resources", "--no-emit-package", "voxcpm"
    )
    run(
        "uv",
        "pip",
        "install",
        "--python",
        str(build_python),
        "--require-hashes",
        "--no-deps",
        "--only-binary",
        ":all:",
        "-r",
        str(build_requirements),
        env=environment,
    )
    packages = runtime / "packages"
    run(
        "uv",
        "pip",
        "install",
        "--python",
        str(build_python),
        "--target",
        str(packages),
        "--require-hashes",
        "--no-deps",
        "--no-build-isolation",
        "-r",
        str(requirements),
        env=environment,
    )
    lock = tomllib.loads((PROJECT / "uv.lock").read_text())
    voxcpm = next(package for package in lock["package"] if package["name"] == "voxcpm")
    origin = voxcpm["source"]["git"]
    url, commit = origin.split("#")
    url = url.split("?")[0]
    if (
        url != "https://github.com/OpenBMB/VoxCPM.git"
        or len(commit) != 40
        or any(char not in "0123456789abcdef" for char in commit)
    ):
        raise ValueError("VoxCPM source must remain an exact reviewed upstream commit")
    source = scratch / "voxcpm-source"
    run("git", "init", str(source))
    run("git", "-C", str(source), "fetch", "--depth=1", url, commit)
    run("git", "-C", str(source), "checkout", "--detach", "FETCH_HEAD")
    if run("git", "-C", str(source), "rev-parse", "HEAD", capture=True) != commit:
        raise ValueError("VoxCPM source commit mismatch")
    source_archive = scratch / "voxcpm-source.tar"
    run("git", "-C", str(source), "archive", "--format=tar", "HEAD", "-o", str(source_archive))
    wheelhouse = scratch / "wheelhouse"
    environment["SETUPTOOLS_SCM_PRETEND_VERSION_FOR_VOXCPM"] = voxcpm["version"]
    run(
        "uv",
        "build",
        "--wheel",
        "--no-build-isolation",
        "--python",
        str(build_python),
        "--out-dir",
        str(wheelhouse),
        str(source),
        env=environment,
    )
    wheels = list(wheelhouse.glob("*.whl"))
    if len(wheels) != 1:
        raise ValueError("VoxCPM build must produce exactly one wheel")
    run(
        "uv",
        "pip",
        "install",
        "--python",
        str(build_python),
        "--target",
        str(packages),
        "--no-index",
        "--no-deps",
        str(wheels[0]),
        env=environment,
    )
    evidence = scratch / "build-evidence"
    write_inventory(runtime, PROJECT / "uv.lock", evidence, PROJECT / "runtime-source.json", host_target())
    shutil.copy2(requirements, evidence / "requirements.txt")
    shutil.copy2(requirements, runtime / "runtime-requirements.txt")
    shutil.copy2(build_requirements, evidence / "build-requirements.txt")
    shutil.copy2(source_archive, evidence / "voxcpm-source.tar")
    compiled_sources = json.loads((PROJECT / "compiled-source-evidence.json").read_text())
    sources_directory = evidence / "compiled-sources"
    sources_directory.mkdir()
    for dependency_source in compiled_sources["sources"]:
        download(dependency_source, sources_directory / dependency_source["fileName"])
    shutil.copy2(PROJECT / "compiled-source-evidence.json", evidence / "compiled-source-evidence.json")
    return {
        "commit": commit,
        "tree": run("git", "-C", str(source), "rev-parse", "HEAD^{tree}", capture=True),
        "sourceSha256": digest(source_archive),
        "wheelSha256": digest(wheels[0]),
        "requirementsSha256": digest(requirements),
        "buildRequirementsSha256": digest(build_requirements),
    }


def relocated_smoke(runtime: Path, target: str, version: str) -> None:
    executable = runtime / ("python.exe" if target == "windows-x64" else "bin/python3")
    code = (
        "import sys; sys.path.insert(0, sys.argv[1]); "
        "assert '.'.join(map(str,sys.version_info[:3]))==sys.argv[2]; "
        "import torch,onnxruntime,sherpa_onnx,voxcpm,sentence_transformers,huggingface_hub,modelscope_hub; "
        "from pyannote.audio import Pipeline; from importlib.metadata import version; print(version('torch'))"
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
    engine_import = "import mlx_whisper" if target == "macos-arm64" else "import faster_whisper"
    run(
        str(executable),
        *ISOLATED_PYTHON_ARGS,
        "-c",
        "import sys;sys.path.insert(0,sys.argv[1]);" + engine_import,
        str(runtime / "packages"),
        cwd=runtime.parent,
        env=environment,
    )


def build(output: Path, signing: str, global_archive_url: str, china_archive_url: str) -> None:
    for archive_url in (global_archive_url, china_archive_url):
        url = urlparse(archive_url)
        if url.scheme != "https" or not url.hostname or url.username or url.password or url.fragment:
            raise ValueError("Runtime publication requires immutable global and China HTTPS archive URLs")
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
        dependencies = install_dependencies(runtime, scratch, executable)
        relocated = scratch / "relocated runtime 中文"
        runtime.rename(relocated)
        relocated_smoke(relocated, target, config["version"])
        signature = sign_runtime(relocated, target, signing)
        relocated_smoke(relocated, target, config["version"])
        record = {
            "schemaVersion": 1,
            "target": target,
            "version": config["version"],
            "standaloneRelease": config["standaloneRelease"],
            "interpreterSource": source,
            "uvVersion": actual_uv,
            "dependencyLockSha256": digest(PROJECT / "uv.lock"),
            "appCommit": run("git", "rev-parse", "HEAD", cwd=REPOSITORY, capture=True),
            "voxcpm": dependencies,
            "relocationVerified": True,
            "signing": signature,
            "licenseReview": "required",
        }
        (scratch / "build-evidence/producer.json").write_text(json.dumps(record, indent=2) + "\n")
        run(
            "node",
            str(REPOSITORY / "scripts/runtime/write-manifest.mjs"),
            "--target",
            target,
            "--directory",
            str(relocated),
        )
        publication = scratch / "publication"
        publication.mkdir()
        artifact = publication / f"python-runtime-{target}.tar.gz"
        archive_runtime(relocated, artifact)
        (publication / f"{artifact.name}.sha256").write_text(digest(artifact) + "\n")
        shutil.copytree(scratch / "build-evidence", publication / "build-evidence")
        evidence_archive = publication / f"python-runtime-{target}-evidence.tar.gz"
        archive_runtime(publication / "build-evidence", evidence_archive)
        (publication / f"{evidence_archive.name}.sha256").write_text(digest(evidence_archive) + "\n")
        manifest = json.loads((relocated / "runtime-manifest.json").read_text())
        descriptor = {
            "schemaVersion": 1,
            "target": target,
            "version": config["version"],
            "dependencyRequirementsSha256": manifest["dependencyRequirementsSha256"],
            "voxcpmCommit": dependencies["commit"],
            "sources": {"global": global_archive_url, "china": china_archive_url},
            "size": artifact.stat().st_size,
            "sha256": digest(artifact),
            "manifestSha256": digest(relocated / "runtime-manifest.json"),
        }
        (publication / "runtime-descriptor.json").write_text(json.dumps(descriptor, indent=2) + "\n")
        publication.rename(output)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--signing", choices=("adhoc", "unsigned", "production"), required=True)
    parser.add_argument("--global-archive-url", required=True, help="Immutable global HTTPS location of the archive")
    parser.add_argument("--china-archive-url", required=True, help="Immutable China HTTPS location of the same archive")
    args = parser.parse_args()
    build(args.output.resolve(), args.signing, args.global_archive_url, args.china_archive_url)
