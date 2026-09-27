import { z } from "zod";

import { SPACE_NODE_KINDS } from "../model/nodes/node-kind";
import { packageTimeSchema } from "./scalar-schema";

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
  .strict();
