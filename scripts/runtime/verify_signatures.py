from __future__ import annotations

import argparse
import sys
from pathlib import Path


def main() -> None:
    # Isolated Python excludes scripts/ from sys.path; authorize this tool package only.
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    from runtime.signing import verify_runtime

    parser = argparse.ArgumentParser(description="Verify production runtime signatures against approved identities")
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--target", choices=("macos-arm64", "windows-x64"), required=True)
    args = parser.parse_args()
    verify_runtime(args.directory, args.target)


if __name__ == "__main__":
    main()
