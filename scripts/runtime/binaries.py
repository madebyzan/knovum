"""Identify compiled runtime files for inventory and signature checks."""

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


def compiled_binaries(root: Path, target: str) -> list[Path]:
    files = []
    for path in sorted(root.rglob("*")):
        if path.is_file():
            with path.open("rb") as source:
                magic = source.read(4)
            if (target.startswith("macos-") and magic in MACHO) or (target == "windows-x64" and magic[:2] == b"MZ"):
                files.append(path)
    if not files:
        raise ValueError("Runtime contains no compiled executable files")
    return files
