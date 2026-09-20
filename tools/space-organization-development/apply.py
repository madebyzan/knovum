"""Reviewed development-only edits. No runtime compatibility or repair path is added."""

from pathlib import Path
import subprocess

root = Path("src/features/spaces")
changed = []


def replace(path: str, before: str, after: str) -> None:
    target = root / path
    text = target.read_text()
    if before in text:
        target.write_text(text.replace(before, after))
        changed.append(str(target))
    elif after not in text:
        raise RuntimeError(f"The reviewed source precondition changed: {path}")


# Remove repeated import text left by a rerun of the previous development formatter.
test = root / "__tests__/canvas-document-workspace.test.ts"
text = test.read_text()
line = 'import type { SaveCanvasStateRequest } from "../canvas/contract/canvas-persistence";'
lines = text.splitlines()
found = False
unique = []
for candidate in lines:
    if candidate == line:
        if found:
            continue
        found = True
    unique.append(candidate)
if not found:
    unique.insert(3, line)
test.write_text("\n".join(unique) + "\n")
changed.append(str(test))

replace("session/actions/canvas-selection-actions.ts",
        '    if (store.activeSpaceId !== spaceId) throw new Error("Canvas deletion target changed");\n', '')
replace("session/actions/canvas-selection-actions.ts",
        '    function applyDeletedState() {\n',
        '    function applyDeletedState() {\n      if (store.activeSpaceId !== spaceId) return;\n')
replace("session/actions/canvas-selection-actions.ts",
        '      const nodes = [...store.nodes];\n      for (const { node, index } of removedNodes) nodes.splice(Math.min(index, nodes.length), 0, node);\n      store.nodes = applyPayloads(nodes, survivorBefore);\n      store.relations = [...store.relations, ...removedRelations];',
        '      if (store.activeSpaceId === spaceId) {\n        const nodes = [...store.nodes];\n        for (const { node, index } of removedNodes) nodes.splice(Math.min(index, nodes.length), 0, node);\n        store.nodes = applyPayloads(nodes, survivorBefore);\n        store.relations = [...store.relations, ...removedRelations];\n      }')
replace("session/actions/json-canvas-import-actions.ts",
        '    if (context.state.activeSpaceId.value !== spaceId) {\n      completeJsonCanvasPreparation(preparation);\n      throw new Error("JSON Canvas import target changed");\n    }\n', '')
replace("session/actions/json-canvas-import-actions.ts",
        '    context.state.nodes.value = [...context.state.nodes.value, ...imported];\n    context.state.relations.value = [...context.state.relations.value, ...committed.importedRelations];\n    context.mutations.commitPendingItems(spaceId, imported, "none");\n    context.placement.requestReveal(imported.map(({ id }) => id));',
        '    if (context.state.activeSpaceId.value === spaceId) {\n      context.state.nodes.value = [...context.state.nodes.value, ...imported];\n      context.state.relations.value = [...context.state.relations.value, ...committed.importedRelations];\n      context.mutations.commitPendingItems(spaceId, imported, "none");\n      context.placement.requestReveal(imported.map(({ id }) => id));\n    }')
replace("__tests__/canvas-session-commit-guard.test.ts",
        'await expect(transaction).rejects.toThrow("JSON Canvas import target changed");',
        'await expect(transaction).resolves.toMatchObject({ revision: 1 });')
replace("__tests__/canvas-session-commit-guard.test.ts",
        'await expect(transaction).rejects.toThrow("Canvas deletion target changed");',
        'await expect(transaction).resolves.toMatchObject({ revision: 1 });')

# The script itself is single-purpose scaffolding; format only its real source edits.
if changed:
    subprocess.run(["pnpm", "exec", "oxfmt", *dict.fromkeys(changed)], check=True)
