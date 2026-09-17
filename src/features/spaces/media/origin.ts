import { z } from "zod";

import { httpUrlSchema } from "../model/nodes/schema/content";

export const spaceMediaOriginSchema = z
  .object({
    kind: z.literal("web"),
    url: httpUrlSchema,
    provider: z.string().trim().min(1).max(128),
    externalId: z.string().trim().min(1).max(512),
  })
  .strict();

export type SpaceMediaOrigin = z.infer<typeof spaceMediaOriginSchema>;
