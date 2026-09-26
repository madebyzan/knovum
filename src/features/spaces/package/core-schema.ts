import { z } from "zod";

import { KNOWLEDGE_OBJECT_STATUSES } from "../knowledge/model/object-types";
import { SPACE_NODE_KINDS } from "../model/nodes/node-kind";
import type { SpaceRelation } from "../model/relations/relation";
import { SPACE_RELATION_KINDS, SPACE_RELATION_STATUSES } from "../model/relations/relation";

export const packageTimeSchema = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);
export const packageIdSchema = z.string().min(1).max(256);
export const relationKindSchema = z.enum(SPACE_RELATION_KINDS);
const relationStatusSchema = z.enum(SPACE_RELATION_STATUSES);
export const objectStatusSchema = z.enum(KNOWLEDGE_OBJECT_STATUSES);
const queryFilterSchema = z
  .object({
    text: z.string().max(512).optional(),
    tags: z.array(z.string().min(1).max(80)).max(64).optional(),
    objectTypeIds: z.array(z.string().min(1).max(128)).max(64).optional(),
    statuses: z.array(objectStatusSchema).max(32).optional(),
    relationKinds: z.array(relationKindSchema).max(32).optional(),
    relationStatuses: z.array(relationStatusSchema).max(8).optional(),
  })
  .strict();
export const spaceSchema = z
  .object({
    id: z.string().min(1),
    kind: z.enum(["inbox", "space"]),
    title: z.string().min(1).max(512),
    archivedAt: packageTimeSchema.nullable(),
    lastOpenedAt: packageTimeSchema,
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict();
export const nodeSchema = z
  .object({
    id: z.string().min(1),
    kind: z.enum(SPACE_NODE_KINDS),
    data: z.unknown(),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict()
  .refine(({ kind }) => String(kind) !== "section", {
    message: "Legacy Section content nodes are not supported",
    path: ["kind"],
  });
export const sceneSchema = z
  .object({
    id: z.string().min(1),
    spaceId: z.string().min(1),
    title: z.string().min(1).max(512),
    viewportX: z.number().finite(),
    viewportY: z.number().finite(),
    viewportZoom: z.number().finite().positive(),
    orderIndex: z.number().int(),
    focusStructureNodeIds: z.array(z.string()).max(100_000),
    relationKinds: z.array(relationKindSchema).max(32),
    relationStatuses: z.array(relationStatusSchema).max(8),
    queryFilter: queryFilterSchema,
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict();
const relationShape = {
  id: z.string().min(1),
  spaceId: z.string().min(1),
  sourceId: z.string().min(1),
  targetId: z.string().min(1),
  kind: relationKindSchema,
  rationale: z.string().max(16_000),
  status: relationStatusSchema,
  statusBeforeInvalidation: z.enum(["active", "contested"]).nullable(),
  confidence: z.number().min(0).max(1).nullable(),
  actorKind: z.enum(["human", "assistant", "agent", "system"]),
  actorId: z.string().nullable(),
  createdAt: packageTimeSchema,
  updatedAt: packageTimeSchema,
};

/** Invalidation is only reachable from a recorded prior status. */
function validateRelationLifecycle(
  value: { status: string; statusBeforeInvalidation: string | null },
  context: z.RefinementCtx,
) {
  const valid =
    value.status === "invalidated"
      ? value.statusBeforeInvalidation === "active" || value.statusBeforeInvalidation === "contested"
      : value.statusBeforeInvalidation === null;
  if (!valid)
    context.addIssue({
      code: "custom",
      message: "Relation lifecycle state is invalid",
      path: ["statusBeforeInvalidation"],
    });
}

export const relationSchema = z.object(relationShape).strict().superRefine(validateRelationLifecycle);

/** Copied Relations come from the domain, which omits absent optionals instead of storing null. */
export const clipboardRelationSchema = z
  .object({
    ...relationShape,
    statusBeforeInvalidation: relationShape.statusBeforeInvalidation.default(null),
    confidence: relationShape.confidence.default(null),
    actorId: relationShape.actorId.default(null),
  })
  .strict()
  .superRefine(validateRelationLifecycle);

/** Durable records carry explicit nulls; the Space domain models the same absence as an omitted key. */
export function toSpaceRelation({
  confidence,
  actorId,
  ...relation
}: z.infer<typeof relationSchema>): SpaceRelation {
  return {
    ...relation,
    ...(confidence === null ? {} : { confidence }),
    ...(actorId === null ? {} : { actorId }),
  };
}
