import { z } from "zod";

import { CANVAS_RELATION_OVERRIDE_COLORS } from "../canvas/contract/canvas-document";
import { structureLayoutSchema } from "../canvas/contract/structure-layout";
import { objectFieldSchema } from "../knowledge/model/object-field-schema";
import { spaceNodePayloadSchema } from "../model/nodes/schema/node-variants";
import { SPACE_RELATION_KINDS } from "../model/relations/relation";
import { spaceStructureSchema } from "../model/structure/structure-schema";
import { validateStructureSnapshot } from "../organization/structure/structure-snapshot-validation";

const id = z.string().min(1).max(128);
export const spaceClipboardPayloadSchema = z
  .object({
    version: z.literal(1),
    kind: z.literal("space-selection"),
    sourceSpaceId: id,
    structure: spaceStructureSchema,
    layout: structureLayoutSchema,
    nodes: z.array(z.object({ id, data: spaceNodePayloadSchema }).strict()).max(10_000),
    objectTypes: z
      .array(
        z
          .object({
            id,
            name: z.string().trim().min(1).max(128),
            icon: id,
            description: z.string().max(4096),
            fields: z.array(objectFieldSchema).max(64),
          })
          .strict(),
      )
      .max(10_000),
    relations: z
      .array(
        z
          .object({
            id,
            sourceId: id,
            targetId: id,
            kind: z.enum(SPACE_RELATION_KINDS),
            rationale: z.string().max(16_000),
            status: z.enum(["active", "contested"]),
            confidence: z.number().finite().min(0).max(1).optional(),
            color: z.enum(CANVAS_RELATION_OVERRIDE_COLORS).optional(),
          })
          .strict(),
      )
      .max(100_000),
  })
  .strict();

export function parseSpaceClipboardPayload(value: unknown) {
  const payload = spaceClipboardPayloadSchema.parse(value);
  if (!payload.structure.items.length || payload.structure.items.length > 10_000)
    throw new Error("A clipboard selection must contain 1 to 10,000 structural items");
  validateStructureSnapshot(payload);
  const placed = new Set(
    payload.structure.items.flatMap((item) => (item.kind === "content" ? [item.nodeId] : [])),
  );
  const nodes = new Set(payload.nodes.map((node) => node.id));
  if (
    placed.size !== nodes.size ||
    nodes.size !== payload.nodes.length ||
    [...placed].some((id) => !nodes.has(id))
  )
    throw new Error("Clipboard content must match its structure exactly");
  const relations = new Set<string>();
  for (const relation of payload.relations) {
    if (!nodes.has(relation.sourceId) || !nodes.has(relation.targetId) || relations.has(relation.id))
      throw new Error("Clipboard relationships must be unique and internal to the selection");
    relations.add(relation.id);
  }
  const types = new Set(payload.objectTypes.map((type) => type.id));
  if (types.size !== payload.objectTypes.length || [...types].some((id) => id.startsWith("system:")))
    throw new Error("Clipboard custom types must be unique and non-system");
  for (const { data } of payload.nodes) {
    if (data.kind === "object" && !data.objectTypeId.startsWith("system:") && !types.has(data.objectTypeId))
      throw new Error("Clipboard is missing a required custom object type");
  }
  return payload;
}
