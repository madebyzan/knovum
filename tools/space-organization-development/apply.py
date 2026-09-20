"""Capture the exact committed source for offline verification; do not rewrite product code."""
import json
import os
from pathlib import Path
import subprocess

if os.environ.get("GITHUB_REF") != "refs/heads/refactor/space-organization":
    raise SystemExit("This development checkpoint is restricted to the organization branch")

out = Path(os.environ["RUNNER_TEMP"]) / "organization-report"
out.mkdir(parents=True, exist_ok=True)
identity = {
    "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
    "tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], text=True).strip(),
}
subprocess.run(["git", "archive", "--format=tar.gz", f"--output={out / 'source.tar.gz'}", "HEAD"], check=True)
(out / "source-identity.json").write_text(json.dumps(identity, indent=2) + "\n")
print("Captured committed source", identity["commit"], "without modifying the worktree")
