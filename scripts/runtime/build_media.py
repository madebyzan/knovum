"""Produce the frozen core FFmpeg resource on its target host."""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path
from typing import cast
from urllib.parse import urlparse

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from runtime.artifacts import archive_runtime, digest
from runtime.commands import run
from runtime.host import host_target
from runtime.media_binary import install_static_binary, require_static_binary
from runtime.signing import sign_runtime

REPOSITORY = Path(__file__).resolve().parents[2]
SOURCE_PATH = REPOSITORY / "scripts/runtime/media-source.json"
LICENSE_REVIEW_PATH = REPOSITORY / "scripts/runtime/media-license-review.json"
CONFIGURATION = json.loads(SOURCE_PATH.read_text())
VERSION = CONFIGURATION["version"]


def license_review(target: str, signing: str) -> dict[str, str]:
    arguments = [
        "node",
        str(REPOSITORY / "scripts/runtime/media-license-policy.mjs"),
        str(REPOSITORY),
        target,
    ]
    if signing != "production":
        arguments.append("--allow-pending")
    review = cast("dict[str, str]", json.loads(run(*arguments, capture=True)))
    if review["status"] == "pending":
        print(
            "Core media license review is pending; development output is not release-approved",
            file=sys.stderr,
        )
    return review


def relocated_smoke(root: Path, target: str) -> None:
    executable = root / ("bin/ffmpeg.exe" if target == "windows-x64" else "bin/ffmpeg")
    version = run(str(executable), "-version", cwd=root.parent, capture=True)
    if not version.startswith((f"ffmpeg version {VERSION} ", f"ffmpeg version {VERSION}-")):
        raise ValueError("Core FFmpeg version differs from the frozen resource")
    configuration = run(str(executable), "-hide_banner", "-buildconf", capture=True)
    if "--enable-nonfree" in configuration:
        raise ValueError("Core FFmpeg must remain redistributable")
    filters = run(str(executable), "-hide_banner", "-filters", capture=True)
    if " subtitles " not in filters:
        raise ValueError("Core FFmpeg requires libass subtitle rendering")
    if any(f" {name} " not in filters for name in ("astats", "loudnorm", "concat")):
        raise ValueError("Core FFmpeg requires audio analysis, loudness and concatenation filters")
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
    with tempfile.TemporaryDirectory(prefix="knovum-media-smoke-", dir=root.parent) as folder:
        sample = Path(folder)
        (sample / "captions.ass").write_text(
            "[Script Info]\nScriptType: v4.00+\nPlayResX: 64\nPlayResY: 64\n"
            "[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, Alignment\n"
            "Style: Default,Arial,12,&H00FFFFFF,2\n"
            "[Events]\nFormat: Layer, Start, End, Style, Text\n"
            "Dialogue: 0,0:00:00.00,0:00:00.20,Default,Qualification\n",
            encoding="utf-8",
        )
        run(
            str(executable),
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-f",
            "lavfi",
            "-i",
            "color=s=64x64:r=10:d=0.1",
            "-f",
            "lavfi",
            "-i",
            "color=s=64x64:r=10:d=0.1",
            "-filter_complex",
            "[0:v][1:v]concat=n=2:v=1:a=0,subtitles=captions.ass[v]",
            "-map",
            "[v]",
            "-c:v",
            "libx264",
            "-f",
            "null",
            "-",
            cwd=sample,
        )
        run(
            str(executable),
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-f",
            "lavfi",
            "-i",
            "sine=duration=0.1",
            "-af",
            "astats",
            "-c:a",
            "pcm_s16le",
            "-f",
            "null",
            "-",
            cwd=sample,
        )


def build(output: Path, signing: str, archive_url: str | None) -> None:
    if output.exists():
        raise ValueError("Core media output already exists; use a fresh destination")
    if archive_url:
        url = urlparse(archive_url)
        if url.scheme != "https" or not url.hostname or url.username or url.password or url.fragment:
            raise ValueError("Core media publication requires an immutable HTTPS archive URL")
    target = host_target()
    review = license_review(target, signing)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="knovum-media-", dir=output.parent) as directory:
        scratch = Path(directory)
        runtime = scratch / "runtime"
        executable = install_static_binary(CONFIGURATION, target, scratch, runtime)
        require_static_binary(executable, target)
        relocated = scratch / "relocated media 中文"
        runtime.rename(relocated)
        signature = sign_runtime(relocated, target, signing)
        relocated_smoke(relocated, target)
        record = {
            "schemaVersion": 1,
            "target": target,
            "version": VERSION,
            "source": CONFIGURATION["targets"][target],
            "compiledBinaryFiles": ["bin/ffmpeg.exe" if target == "windows-x64" else "bin/ffmpeg"],
            "appCommit": run("git", "rev-parse", "HEAD", cwd=REPOSITORY, capture=True),
            "relocationVerified": True,
            "signing": signature,
            "licenseReview": review,
        }
        evidence = relocated / "build-evidence"
        evidence.mkdir()
        shutil.copy2(SOURCE_PATH, evidence / "media-source.json")
        shutil.copy2(LICENSE_REVIEW_PATH, evidence / "media-license-review.json")
        for name in (
            "artifacts.py",
            "binaries.py",
            "build_media.py",
            "media_binary.py",
            "media-license-policy.mjs",
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
            "sourceSha256": CONFIGURATION["targets"][target]["sha256"],
            "licenseReviewSha256": review["sha256"],
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
