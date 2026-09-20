import { isEqual } from "es-toolkit";

import { applyStructureChanges, diffStructureSnapshots } from "../../organization/structure/commands/changes";
import { parseSpaceEditorDocument, type SpaceEditorDocument } from "../contract/canvas-document";

/** Replays exact committed records, preserving the live camera and disjoint later edits. */
export function replayEditorTransition(
  current: SpaceEditorDocument,
  before: SpaceEditorDocument,
  after: SpaceEditorDocument,
) {
  const snapshot = applyStructureChanges(current, diffStructureSnapshots(before, after));
  const previous = new Map(before.relationStyles?.map((style) => [style.relationId, style]));
  const target = new Map(after.relationStyles?.map((style) => [style.relationId, style]));
  const styles = new Map(current.relationStyles?.map((style) => [style.relationId, style]));
  for (const id of new Set([...previous.keys(), ...target.keys()])) {
    if (isEqual(previous.get(id), target.get(id))) continue;
    if (!isEqual(styles.get(id), previous.get(id)))
      throw new Error("Relation appearance changed after this edit");
    const next = target.get(id);
    if (next) styles.set(id, structuredClone(next));
    else styles.delete(id);
  }
  return parseSpaceEditorDocument({ ...current, ...snapshot, relationStyles: [...styles.values()] });
}
