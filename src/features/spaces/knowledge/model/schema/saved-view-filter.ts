import { z } from "zod";

import { relationKinds, relationStatuses } from "../../../contracts/generated/contract-values";
import { KNOWLEDGE_OBJECT_STATUSES } from "../object-types";

export const savedViewFilterSchema = z
  .object({
    text: z.string().max(512).optional(),
    tags: z.array(z.string().min(1).max(80)).max(64).optional(),
    objectTypeIds: z.array(z.string().min(1).max(128)).max(64).optional(),
    statuses: z.array(z.enum(KNOWLEDGE_OBJECT_STATUSES)).max(32).optional(),
    relationKinds: z.array(z.enum(relationKinds)).max(32).optional(),
    relationStatuses: z.array(z.enum(relationStatuses)).max(8).optional(),
  })
  .strict();
