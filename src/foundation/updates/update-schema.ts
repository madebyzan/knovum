import { z } from "zod";

import type { UpdateSnapshot } from "../../shared/contracts/generated/UpdateSnapshot";

const integer = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);
const release = z.strictObject({
  version: z.string().min(1).max(128),
  currentVersion: z.string().min(1).max(128),
  notes: z.string().max(65_536),
});

export const updateSnapshotSchema: z.ZodType<UpdateSnapshot> = z.strictObject({
  revision: integer,
  state: z.discriminatedUnion("status", [
    z.strictObject({ status: z.literal("disabled") }),
    z.strictObject({ status: z.literal("idle") }),
    z.strictObject({ status: z.literal("checking") }),
    z.strictObject({ status: z.literal("current") }),
    z.strictObject({ status: z.literal("available"), release }),
    z.strictObject({ status: z.literal("downloading"), release, downloaded: integer, total: integer.nullable() }),
    z.strictObject({ status: z.literal("verifying"), release }),
    z.strictObject({ status: z.literal("ready"), release }),
    z.strictObject({ status: z.literal("installing"), release }),
    z.strictObject({ status: z.literal("failed"), reason: z.enum(["check", "download", "signature", "install"]) }),
  ]),
});
