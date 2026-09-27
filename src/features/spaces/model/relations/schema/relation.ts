import { z } from "zod";

import { SPACE_RELATION_KINDS, SPACE_RELATION_STATUSES, type SpaceRelation } from "../relation";

const relationShape = {
  id: z.string().min(1),
  spaceId: z.string().min(1),
  sourceId: z.string().min(1),
  targetId: z.string().min(1),
  kind: z.enum(SPACE_RELATION_KINDS),
  rationale: z.string().max(16_000),
  status: z.enum(SPACE_RELATION_STATUSES),
  statusBeforeInvalidation: z.enum(["active", "contested"]).nullable(),
  confidence: z.number().min(0).max(1).nullable(),
  actorKind: z.enum(["human", "assistant", "agent", "system"]),
  actorId: z.string().nullable(),
  createdAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
  updatedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
};

/** Invalidation is only reachable from a recorded prior status. */
export function validateRelationLifecycle(
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
