import { z } from "zod";

import type { ObjectStorageSettings } from "../../../shared/contracts/generated/ObjectStorageSettings";

const INVALID_HTTPS_URL = "Must be an https:// URL with a host";
const INVALID_BUCKET = "Bucket must be 3-63 characters of lowercase letters, numbers, '.' or '-'";

function httpsUrlSchema() {
  return z
    .string()
    .trim()
    .min(1)
    .superRefine((value, context) => {
      if (!URL.canParse(value)) {
        context.addIssue({ code: "custom", message: INVALID_HTTPS_URL });
        return;
      }
      const url = new URL(value);
      if (url.protocol !== "https:" || !url.hostname || url.username || url.password) {
        context.addIssue({ code: "custom", message: INVALID_HTTPS_URL });
      }
    });
}

export const objectStorageSettingsSchema: z.ZodType<ObjectStorageSettings> = z
  .object({
    endpoint: httpsUrlSchema(),
    region: z.string().trim().min(1),
    bucket: z
      .string()
      .trim()
      .min(3)
      .max(63)
      .regex(/^[a-z0-9][a-z0-9.-]*[a-z0-9]$/, INVALID_BUCKET)
      .refine((value) => !value.includes(".."), INVALID_BUCKET),
    publicBaseUrl: z.nullable(httpsUrlSchema()),
  })
  .strict();
