import { z } from "zod";

import { savedViewFilterSchema } from "../../knowledge/model/schema/saved-view-filter";
import { SPACE_RELATION_KINDS, SPACE_RELATION_STATUSES } from "../../model/relations/relation";

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
    relationKinds: z.array(z.enum(SPACE_RELATION_KINDS)).max(32),
    relationStatuses: z.array(z.enum(SPACE_RELATION_STATUSES)).max(8),
    queryFilter: savedViewFilterSchema,
    createdAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    updatedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
  })
  .strict();
