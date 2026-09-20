import { z } from "zod";

import { itemIdSchema, MAX_STRUCTURE_ITEMS } from "../../../model/structure/structure-schema";
import { structurePointSchema, structureSizeSchema, structureTransformSchema } from "../../../canvas/contract/structure-layout";

const ids = z.array(itemIdSchema).min(1).max(MAX_STRUCTURE_ITEMS);
const title = z.string().trim().min(1).max(512);
const bounds = structurePointSchema.extend(structureSizeSchema.shape).strict();
const parentId = itemIdSchema.nullable();
const insertIndex = z.number().int().nonnegative().max(MAX_STRUCTURE_ITEMS).optional();
export const structureCommandSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("create-empty-section"), id: itemIdSchema, title, bounds }).strict(),
  z.object({ kind: z.literal("draw-section"), id: itemIdSchema, title, bounds, ids: ids.min(0) }).strict(),
  z.object({ kind: z.literal("section-around-selection"), id: itemIdSchema, title, ids }).strict(),
  z.object({ kind: z.literal("group-selection"), id: itemIdSchema, title, ids }).strict(),
  z.object({ kind: z.literal("dissolve-container"), id: itemIdSchema }).strict(),
  z.object({ kind: z.literal("move-items"), ids, delta: structurePointSchema }).strict(),
  z.object({ kind: z.literal("reparent-items"), ids, parentId, insertIndex }).strict(),
  z.object({ kind: z.literal("drop-items"), ids, parentId, delta: structurePointSchema }).strict(),
  z.object({ kind: z.literal("resize-section"), id: itemIdSchema, bounds }).strict(),
  z.object({ kind: z.literal("fit-section"), id: itemIdSchema }).strict(),
  z.object({ kind: z.literal("set-section-collapsed"), id: itemIdSchema, collapsed: z.boolean() }).strict(),
  z.object({ kind: z.literal("rename-container"), id: itemIdSchema, title }).strict(),
  z.object({ kind: z.literal("set-items-locked"), ids, locked: z.boolean() }).strict(),
  z.object({ kind: z.literal("delete-subtrees"), ids }).strict(),
  z.object({ kind: z.literal("transform-group"), id: itemIdSchema, worldTransform: structureTransformSchema }).strict(),
  z.object({ kind: z.literal("align-items"), ids, axis: z.enum(["x", "y"]), alignment: z.enum(["start", "center", "end"]) }).strict(),
  z.object({ kind: z.literal("distribute-items"), ids, axis: z.enum(["x", "y"]) }).strict(),
  z.object({ kind: z.literal("clone-subtrees"), ids, parentId, insertIndex, delta: structurePointSchema,
    identities: z.array(z.object({ sourceId: itemIdSchema, targetId: itemIdSchema }).strict()).min(1).max(MAX_STRUCTURE_ITEMS),
  }).strict(),
]);
export type StructureCommand = z.infer<typeof structureCommandSchema>;
export type StructureCommandOf<K extends StructureCommand["kind"]> = Extract<StructureCommand, { kind: K }>;
