import { z } from "zod";

import { SPACE_NODE_KINDS } from "../model/nodes/node-kind";

export const packageTimeSchema = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);
export const packageIdSchema = z.string().min(1).max(256);
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
