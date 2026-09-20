"""Apply reviewed integration stages on the existing organization branch only."""
import os
from pathlib import Path
import subprocess
import sys

if os.environ.get("GITHUB_REF") != "refs/heads/refactor/space-organization":
    raise SystemExit("This development runner is restricted to the organization branch")

for name in ("native_bindings.py", "package_integration.py", "interface_integration.py", "fragment_execution.py", "clipboard_integration.py"):
    subprocess.run([sys.executable, str(Path(__file__).with_name(name))], check=True)

path = Path("src/features/spaces/presentation/use-structure-menu.ts")
source = path.read_text().replace(
    'entry(label("moveToRoot"), () => editor.executeStructureCommand({ kind: "reparent-items", ids, parentId: null }),',
    'entry(label("moveToRoot"), () => editor.executeStructureCommand({ kind: "reparent-items", ids, parentId: null })),',
)
path.write_text(source)
