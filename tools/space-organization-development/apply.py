"""Apply reviewed, branch-local integration changes before running the real gates."""
import os
from pathlib import Path
import runpy

if os.environ.get("GITHUB_REF") != "refs/heads/refactor/space-organization":
    raise SystemExit("This development runner is restricted to the organization branch")

for name in ("native_bindings.py",):
    runpy.run_path(str(Path(__file__).with_name(name)), run_name="__main__")
