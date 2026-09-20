"""Apply reviewed test typing/lifetime corrections, then format this integration step."""

from pathlib import Path
import subprocess

path = Path("src/features/spaces/__tests__/canvas-document-workspace.test.ts")
text = path.read_text()
text = text.replace('import type { CanvasDocumentV1 } from "../canvas/contract/canvas-document";', 'import type { CanvasDocumentV1 } from "../canvas/contract/canvas-document";\nimport type { SaveCanvasStateRequest } from "../canvas/contract/canvas-persistence";')
text = text.replace('const save = vi.fn(async () => 2);', 'const save = vi.fn<(request: SaveCanvasStateRequest) => Promise<number>>(async () => 2);')
text = text.replace('const operation = first.workspace.commit(() => pending.promise);\n    await Promise.resolve();', 'const persist = vi.fn(() => pending.promise);\n    const operation = first.workspace.commit(persist);\n    await vi.waitFor(() => expect(persist).toHaveBeenCalledOnce());')
path.write_text(text)
paths = [
    "src/features/spaces/canvas/host/canvas-document-workspace.ts",
    "src/features/spaces/canvas/host/use-canvas-document-session.ts",
    str(path),
]
subprocess.run(["pnpm", "exec", "oxfmt", *paths], check=True)
