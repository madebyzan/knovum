"""Native signing is a build step, preceding the runtime file manifest."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
from pathlib import Path

MACHO = {
    b"\xfe\xed\xfa\xce",
    b"\xce\xfa\xed\xfe",
    b"\xfe\xed\xfa\xcf",
    b"\xcf\xfa\xed\xfe",
    b"\xca\xfe\xba\xbe",
    b"\xbe\xba\xfe\xca",
    b"\xca\xfe\xba\xbf",
    b"\xbf\xba\xfe\xca",
}


def native_files(root: Path, target: str) -> list[Path]:
    files = []
    for path in sorted(root.rglob("*")):
        if path.is_file():
            with path.open("rb") as source:
                magic = source.read(4)
            if (target == "macos-arm64" and magic in MACHO) or (target == "windows-x64" and magic[:2] == b"MZ"):
                files.append(path)
    if not files:
        raise ValueError("Runtime contains no native executable files")
    return files


def _mac_requirement() -> str:
    identity = os.environ["APPLE_SIGNING_IDENTITY"]
    match = re.fullmatch(r"Developer ID Application: .+ \(([A-Z0-9]{10})\)", identity)
    if match is None:
        raise ValueError("Production runtime requires an approved Developer ID Application identity and Team ID")
    return (
        "anchor apple generic and certificate leaf[subject.CN] = "
        + json.dumps(identity, ensure_ascii=False)
        + " and certificate leaf[subject.OU] = "
        + json.dumps(match[1])
    )


def _windows_thumbprint() -> str:
    identity = os.environ["KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT"].upper()
    if not re.fullmatch(r"[A-F0-9]{40}", identity):
        raise ValueError("Production runtime requires an approved certificate thumbprint")
    return identity


def _verify_native(path: Path, target: str, *, production: bool) -> None:
    if target == "macos-arm64":
        requirement = ["-R", _mac_requirement()] if production else []
        subprocess.run(["codesign", "--verify", "--strict", *requirement, str(path)], check=True)
        return
    subprocess.run([os.environ["KNOVUM_WINDOWS_SIGNTOOL"], "verify", "/pa", "/all", str(path)], check=True)
    identity = _windows_thumbprint()
    script = (
        "$ErrorActionPreference='Stop'; "
        "$s=Get-AuthenticodeSignature -LiteralPath $env:KNOVUM_NATIVE_VERIFY_PATH; "
        "if($s.Status -ne 'Valid' -or $null -eq $s.SignerCertificate){throw 'Invalid native signature'}; "
        "$s.SignerCertificate.Thumbprint"
    )
    result = subprocess.run(
        ["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
        check=True,
        capture_output=True,
        text=True,
        env={**os.environ, "KNOVUM_NATIVE_VERIFY_PATH": str(path)},
    )
    if result.stdout.strip().upper() != identity:
        raise ValueError(f"Runtime signer differs from the approved certificate: {path}")


def verify_runtime(root: Path, target: str) -> int:
    files = native_files(root, target)
    for path in files:
        _verify_native(path, target, production=True)
    return len(files)


def sign_runtime(root: Path, target: str, mode: str) -> dict[str, str | int]:
    files = native_files(root, target)
    if target == "macos-arm64":
        if mode == "unsigned":
            raise ValueError("macOS runtime requires explicit ad-hoc or production signing")
        identity = "-" if mode == "adhoc" else os.environ["APPLE_SIGNING_IDENTITY"]
        if mode == "production":
            _mac_requirement()
        for path in files:
            options = ["--options", "runtime", "--timestamp"] if mode == "production" else []
            subprocess.run(["codesign", "--force", "--sign", identity, *options, str(path)], check=True)
            _verify_native(path, target, production=mode == "production")
    else:
        if mode == "adhoc":
            raise ValueError("Ad-hoc signing is not a Windows signing mode")
        identity = "unsigned"
        if mode == "production":
            tool = os.environ["KNOVUM_WINDOWS_SIGNTOOL"]
            identity = _windows_thumbprint()
            timestamp = os.environ["KNOVUM_WINDOWS_TIMESTAMP_URL"]
            if not timestamp.startswith("https://"):
                raise ValueError("Authenticode timestamp service requires HTTPS")
            for path in files:
                subprocess.run(
                    [tool, "sign", "/sha1", identity, "/fd", "SHA256", "/tr", timestamp, "/td", "SHA256", str(path)],
                    check=True,
                )
                _verify_native(path, target, production=True)
    return {"mode": mode, "identity": identity, "nativeFileCount": len(files)}


def main() -> None:
    parser = argparse.ArgumentParser(description="Verify production runtime signatures against approved identities")
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--target", choices=("macos-arm64", "windows-x64"), required=True)
    args = parser.parse_args()
    verify_runtime(args.directory, args.target)


if __name__ == "__main__":
    main()
