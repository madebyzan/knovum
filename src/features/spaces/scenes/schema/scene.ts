import { z } from "zod";

import { relationKinds, relationStatuses } from "../../contracts/generated/contract-values";
import { savedViewFilterSchema } from "../../knowledge/model/schema/saved-view-filter";

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
    relationKinds: z.array(z.enum(relationKinds)).max(32),
    relationStatuses: z.array(z.enum(relationStatuses)).max(8),
    queryFilter: savedViewFilterSchema,
    createdAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
    updatedAt: z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER),
  })
  .strict();
