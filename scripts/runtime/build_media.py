"""Produce the frozen core FFmpeg resource on its target host."""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path
from urllib.parse import urlparse

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from runtime.artifacts import archive_runtime, digest
from runtime.commands import run
from runtime.conda_packages import install_locked_conda_packages
from runtime.host import host_target
from runtime.media_files import export_media_files
from runtime.signing import sign_runtime

REPOSITORY = Path(__file__).resolve().parents[2]
PROJECT = REPOSITORY / "python"
VERSION = "9.0.1"


def relocated_smoke(root: Path, target: str) -> None:
    executable = root / ("Library/bin/ffmpeg.exe" if target == "windows-x64" else "bin/ffmpeg")
    version = run(str(executable), "-version", cwd=root.parent, capture=True)
    if not version.startswith(f"ffmpeg version {VERSION} "):
        raise ValueError("Core FFmpeg version differs from the frozen resource")
    configuration = run(str(executable), "-hide_banner", "-buildconf", capture=True)
    if "--enable-nonfree" in configuration:
        raise ValueError("Core FFmpeg must remain redistributable")
    filters = run(str(executable), "-hide_banner", "-filters", capture=True)
    if " subtitles " not in filters:
        raise ValueError("Core FFmpeg requires libass subtitle rendering")
    encoders = run(str(executable), "-hide_banner", "-encoders", capture=True)
    encoder = " h264_mf " if target == "windows-x64" else " h264_videotoolbox "
    if encoder not in encoders:
        raise ValueError("Core FFmpeg requires the target platform H.264 encoder")
    # Exercise the actual encoding and loudness filters used by speech/video exports.
    run(
        str(executable),
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-f",
        "lavfi",
        "-i",
        "color=c=black:s=64x64:r=10:d=0.2",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:sample_rate=48000:duration=0.2",
        "-af",
        "loudnorm=I=-18:TP=-1:LRA=11",
        "-c:v",
        "libx264",
        "-c:a",
        "aac",
        "-f",
        "null",
        "-",
        cwd=root.parent,
    )


def build(output: Path, signing: str, archive_url: str | None) -> None:
    if output.exists():
        raise ValueError("Core media output already exists; use a fresh destination")
    if archive_url:
        url = urlparse(archive_url)
        if url.scheme != "https" or not url.hostname or url.username or url.password or url.fragment:
            raise ValueError("Core media publication requires an immutable HTTPS archive URL")
    target = host_target()
    lock = PROJECT / "conda-locks/media" / f"{target}.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="knovum-media-", dir=output.parent) as directory:
        scratch = Path(directory)
        runtime, conda_inventory = install_locked_conda_packages(PROJECT, lock, target, scratch)
        dependency_files = export_media_files(runtime, target)
        relocated = scratch / "relocated media 中文"
        runtime.rename(relocated)
        signature = sign_runtime(relocated, target, signing)
        relocated_smoke(relocated, target)
        record = {
            "schemaVersion": 1,
            "target": target,
            "version": VERSION,
            "condaSupplier": conda_inventory,
            "compiledDependencyFiles": dependency_files,
            "appCommit": run("git", "rev-parse", "HEAD", cwd=REPOSITORY, capture=True),
            "relocationVerified": True,
            "signing": signature,
            "licenseReview": "required",
        }
        evidence = relocated / "build-evidence"
        evidence.mkdir()
        shutil.copy2(lock, evidence / "conda-lock.json")
        for name in (
            "artifacts.py",
            "binaries.py",
            "build_media.py",
            "conda_packages.py",
            "media_files.py",
            "signing.py",
        ):
            shutil.copy2(REPOSITORY / "scripts/runtime" / name, evidence / name)
        producer = scratch / "producer.json"
        producer.write_text(json.dumps(record, indent=2) + "\n")
        run("node", str(REPOSITORY / "scripts/runtime/write-media-manifest.mjs"), str(relocated), target, str(producer))
        publication = scratch / "publication"
        publication.mkdir()
        if archive_url is None:
            relocated.rename(publication / "runtime")
            publication.rename(output)
            try:
                relocated_smoke(output / "runtime", target)
            except BaseException:
                shutil.rmtree(output)
                raise
            return
        artifact = publication / f"ffmpeg-{target}.tar.gz"
        archive_runtime(relocated, artifact)
        descriptor = {
            "schemaVersion": 1,
            "runtimeId": "ffmpeg",
            "target": target,
            "version": VERSION,
            "condaLockSha256": digest(lock),
            "archiveUrl": archive_url,
            "size": artifact.stat().st_size,
            "sha256": digest(artifact),
            "manifestSha256": digest(relocated / "media-manifest.json"),
        }
        (publication / f"ffmpeg-{target}-descriptor.json").write_text(json.dumps(descriptor, indent=2) + "\n")
        publication.rename(output)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--signing", choices=("adhoc", "unsigned", "production"))
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--archive-url")
    mode.add_argument("--development", action="store_true")
    args = parser.parse_args()
    if args.archive_url and not args.signing:
        parser.error("publication requires --signing")
    if args.development and args.signing:
        parser.error("development selects host-local signing; omit --signing")
    signing = args.signing or ("adhoc" if host_target().startswith("macos-") else "unsigned")
    build(args.output.resolve(), signing, args.archive_url)
