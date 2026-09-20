"""One reviewed, idempotent integration step; removed before the final product delivery.

The runner may only fast-forward refactor/space-organization. This file is developer
scaffolding, not runtime schema migration, compatibility code, or a load-time repair.
"""

from pathlib import Path

assert Path("AGENTS.md").is_file()
print("Diagnostic phase: no application changes requested")
