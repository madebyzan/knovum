import { z } from "zod";

import { SPACE_RELATION_KINDS, SPACE_RELATION_STATUSES } from "../../../model/relations/relation";
import { KNOWLEDGE_OBJECT_STATUSES } from "../object-types";

export const savedViewFilterSchema = z
  .object({
    text: z.string().max(512).optional(),
    tags: z.array(z.string().min(1).max(80)).max(64).optional(),
    objectTypeIds: z.array(z.string().min(1).max(128)).max(64).optional(),
    statuses: z.array(z.enum(KNOWLEDGE_OBJECT_STATUSES)).max(32).optional(),
    relationKinds: z.array(z.enum(SPACE_RELATION_KINDS)).max(32).optional(),
    relationStatuses: z.array(z.enum(SPACE_RELATION_STATUSES)).max(8).optional(),
  })
  .strict();
