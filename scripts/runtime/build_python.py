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
from runtime.windows_process import configure_utf8

REPOSITORY = Path(__file__).resolve().parents[2]
PROJECT = REPOSITORY / "python"
ISOLATED_PYTHON_ARGS = ("-I", "-B", "-X", "utf8")


def sign_runtime(root: Path, target: str, mode: str) -> dict[str, str | int]:
    return dict(
        json.loads(
            run(
                "node",
                str(REPOSITORY / "scripts/runtime/compiled-signing.mjs"),
                "--directory",
                str(root),
                "--target",
                target,
                "--mode",
                mode,
                capture=True,
            )
        )
    )


def relocated_smoke(runtime: Path, target: str, version: str, profile: str) -> None:
    executable = runtime / ("python.exe" if target == "windows-x64" else "bin/python3")
    if profile != "agent":
        raise ValueError("Only the Agent Python runtime is retained")
    imports = (
        "from deepagents import create_deep_agent;"
        "from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver;"
        "import langchain_openai,langchain_anthropic,langchain_google_genai;"
        "import langchain_ollama,langchain_deepseek,langchain_groq,truststore"
    )
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
        install_dependencies(runtime, scratch, executable, REPOSITORY, profile, target)
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
            "recipeSha256": run(
                "node",
                str(REPOSITORY / "scripts/runtime/profiles.mjs"),
                "--repository",
                str(REPOSITORY),
                "--target",
                target,
                "--profile",
                profile,
                "--identity",
                capture=True,
            ),
            "appCommit": run("git", "rev-parse", "HEAD", cwd=REPOSITORY, capture=True),
            "relocationVerified": True,
            "signing": signature,
            "licenseReview": "required",
        }
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
        choices=("agent",),
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
