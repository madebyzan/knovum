import { z } from "zod";

import { relationSchema, validateRelationLifecycle } from "../model/relations/relation-schema";

/** Copied Relations come from the domain, which omits absent optionals instead of storing null. */
export const clipboardRelationSchema = z
  .object({
    ...relationSchema.shape,
    statusBeforeInvalidation: relationSchema.shape.statusBeforeInvalidation.default(null),
    confidence: relationSchema.shape.confidence.default(null),
    actorId: relationSchema.shape.actorId.default(null),
  })
  .strict()
  .superRefine(validateRelationLifecycle);
