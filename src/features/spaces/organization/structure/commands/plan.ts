import { requireStructureItem, structureAncestors, structureSubtree } from "../../../model/structure/structure-index";
import { indexStructureSnapshot } from "../structure-context";
import { projectStructure } from "../structure-projection";
import { structureSnapshotSchema, type StructureSnapshot } from "../structure-snapshot-contract";
import { validateStructureSnapshot } from "../structure-snapshot-validation";
import { diffStructureSnapshots, inverseStructureChanges, type StructureChanges } from "./changes";
import { structureCommandSchema, type StructureCommand } from "./contract";
import { createStructureDraft, draftSnapshot } from "./draft";
import { executeStructureCommand } from "./execution";

export interface StructureCommandPlan {
  command: StructureCommand;
  after: StructureSnapshot;
  changes: StructureChanges;
  inverse: StructureChanges;
  selection: readonly string[];
  affectedItemIds: readonly string[];
  effects: { removedContentNodeIds: readonly string[]; contentClones: readonly { sourceNodeId: string; targetNodeId: string }[] };
}
export function planStructureCommand(snapshot: StructureSnapshot, source: StructureCommand, selection: readonly string[] = []): StructureCommandPlan {
  const command = structureCommandSchema.parse(source);
  const context = validateStructureSnapshot(snapshot);
  for (const id of selection) requireStructureItem(context.tree, id);
  const draft = createStructureDraft(snapshot);
  const result = executeStructureCommand({ context, projection: projectStructure(context), draft, selection }, command);
  const after = structureSnapshotSchema.parse(draftSnapshot(draft));
  validateStructureSnapshot(after);
  const changes = diffStructureSnapshots(snapshot, after);
  return { command, after, changes, inverse: inverseStructureChanges(changes), selection: [...result.selection],
    affectedItemIds: affectedIds(snapshot, after, changes), effects: {
      removedContentNodeIds: changes.items.flatMap(({ before, after: next }) => before?.kind === "content" && next === null ? [before.nodeId] : []),
      contentClones: result.contentClones ?? [],
    } };
}
function affectedIds(before: StructureSnapshot, after: StructureSnapshot, changes: StructureChanges) {
  const changed = new Set([...changes.items, ...changes.layouts].map((change) => change.id));
  const affected = new Set(changed);
  for (const snapshot of [before, after]) {
    const { tree } = indexStructureSnapshot(snapshot);
    const present = [...changed].filter((id) => tree.items.has(id));
    for (const item of structureSubtree(tree, present)) affected.add(item.id);
    for (const id of present) for (const parent of structureAncestors(tree, id)) affected.add(parent.id);
  }
  return [...affected];
}
