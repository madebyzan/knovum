import { uniq } from "es-toolkit";
import { z } from "zod";

import { groundedScopeSchema } from "./grounded-scope";
import { knowledgeTimestampSchema } from "./schema";

export const groundedThreadSchema = z
  .object({
    id: z.string().min(1),
    spaceId: z.string().min(1),
    scope: groundedScopeSchema,
    scopeNodeIds: z.array(z.string().min(1)).max(10_000),
    title: z.string().min(1).max(512),
    createdAt: knowledgeTimestampSchema,
    updatedAt: knowledgeTimestampSchema,
  })
  .strict()
  .superRefine((value, context) => {
    if (uniq(value.scopeNodeIds).length !== value.scopeNodeIds.length)
      context.addIssue({
        code: "custom",
        message: "Grounded scope node IDs must be unique",
        path: ["scopeNodeIds"],
      });
    if ((value.scope === "space" || value.scope === "all-spaces") && value.scopeNodeIds.length)
      context.addIssue({
        code: "custom",
        message: "Space-wide scopes cannot contain explicit node IDs",
        path: ["scopeNodeIds"],
      });
  });

export type GroundedThread = z.infer<typeof groundedThreadSchema>;
