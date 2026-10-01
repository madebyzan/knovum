import { z } from "zod";

import type { UpdateSnapshot } from "../../shared/contracts/generated/UpdateSnapshot";

const bytes = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);

export const updateSnapshotSchema: z.ZodType<UpdateSnapshot> = z.strictObject({
  revision: z.number().int().nonnegative(),
  configured: z.boolean(),
  phase: z.enum(["idle", "checking", "available", "downloading", "verifying", "ready", "installing"]),
  update: z
    .strictObject({
      version: z.string().min(1),
      currentVersion: z.string().min(1),
      notes: z.string().nullable(),
    })
    .nullable(),
  downloaded: bytes,
  total: bytes.nullable(),
  failure: z.enum(["network", "signature", "metadata", "installation", "internal"]).nullable(),
});
