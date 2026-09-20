import { isEqual } from "es-toolkit";
import { z } from "zod";

import { structureItemLayoutSchema } from "../../../canvas/contract/structure-layout";
import { StructureError } from "../../../model/structure/structure-error";
import {
  itemIdSchema,
  MAX_STRUCTURE_ITEMS,
  structureItemSchema,
} from "../../../model/structure/structure-schema";
import { structureSnapshotSchema, type StructureSnapshot } from "../structure-snapshot-contract";
import { validateStructureSnapshot } from "../structure-snapshot-validation";
import { createStructureDraft, draftSnapshot } from "./draft";

export const structureChangesSchema = z
  .object({
    items: z
      .array(
        z
          .object({
            id: itemIdSchema,
            before: structureItemSchema.nullable(),
            after: structureItemSchema.nullable(),
          })
          .strict(),
      )
      .max(MAX_STRUCTURE_ITEMS),
    layouts: z
      .array(
        z
          .object({
            id: itemIdSchema,
            before: structureItemLayoutSchema.nullable(),
            after: structureItemLayoutSchema.nullable(),
          })
          .strict(),
      )
      .max(MAX_STRUCTURE_ITEMS),
  })
  .strict();
export type StructureChanges = z.infer<typeof structureChangesSchema>;
interface Change<T> {
  id: string;
  before: T | null;
  after: T | null;
}
export function diffStructureSnapshots(
  before: StructureSnapshot,
  after: StructureSnapshot,
): StructureChanges {
  const left = createStructureDraft(before);
  const right = createStructureDraft(after);
  // History owns its copies; Vue or caller mutation must not rewrite a recorded edit.
  return structureChangesSchema.parse({
    items: diff(left.items, right.items),
    layouts: diff(left.layouts, right.layouts),
  });
}
function diff<T>(before: ReadonlyMap<string, T>, after: ReadonlyMap<string, T>): Change<T>[] {
  return [...new Set([...before.keys(), ...after.keys()])].flatMap((id) => {
    const left = before.get(id) ?? null;
    const right = after.get(id) ?? null;
    return isEqual(left, right) ? [] : [{ id, before: left, after: right }];
  });
}
export function inverseStructureChanges(changes: StructureChanges): StructureChanges {
  return structureChangesSchema.parse({
    items: changes.items.map(({ id, before, after }) => ({ id, before: after, after: before })),
    layouts: changes.layouts.map(({ id, before, after }) => ({ id, before: after, after: before })),
  });
}
export function applyStructureChanges(
  snapshot: StructureSnapshot,
  source: StructureChanges,
): StructureSnapshot {
  const changes = structureChangesSchema.parse(source);
  const draft = createStructureDraft(snapshot);
  applyRecords(draft.items, changes.items, (item) => item.id);
  applyRecords(draft.layouts, changes.layouts, (item) => item.itemId);
  const result = structureSnapshotSchema.parse(draftSnapshot(draft));
  validateStructureSnapshot(result);
  return result;
}
function applyRecords<T>(
  records: Map<string, T>,
  changes: readonly Change<T>[],
  identity: (value: T) => string,
) {
  const seen = new Set<string>();
  for (const change of changes) {
    if (seen.has(change.id))
      throw new StructureError("duplicate-identity", "History cannot change a record twice", [change.id]);
    seen.add(change.id);
    const ids = [change.before, change.after].filter((value): value is T => value !== null).map(identity);
    if (!ids.length || ids.some((id) => id !== change.id))
      throw new StructureError("invalid-reference", "History identity does not match its record", [
        change.id,
      ]);
    if (!isEqual(records.get(change.id) ?? null, change.before))
      throw new StructureError("history-conflict", "A record changed after the history entry", [change.id]);
    if (change.after === null) records.delete(change.id);
    else records.set(change.id, change.after);
  }
}
