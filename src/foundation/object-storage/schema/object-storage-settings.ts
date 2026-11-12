import { z } from "zod";

import type { ObjectStorageSettings } from "../../../shared/contracts/generated/ObjectStorageSettings";

const VALIDATION = "settings.objectStorage.validation";
const BUCKET_INVALID = `${VALIDATION}.bucketInvalid`;

function httpsUrlSchema(requiredKey: string, invalidKey: string) {
  return z
    .string()
    .trim()
    .min(1, { message: requiredKey })
    .superRefine((value, context) => {
      if (!URL.canParse(value)) {
        context.addIssue({ code: "custom", message: invalidKey });
        return;
      }
      const url = new URL(value);
      if (url.protocol !== "https:" || !url.hostname || url.username || url.password) {
        context.addIssue({ code: "custom", message: invalidKey });
      }
    });
}

export const objectStorageSettingsSchema: z.ZodType<ObjectStorageSettings> = z
  .object({
    endpoint: httpsUrlSchema(`${VALIDATION}.endpointRequired`, `${VALIDATION}.endpointInvalid`),
    region: z
      .string()
      .trim()
      .min(1, { message: `${VALIDATION}.regionRequired` }),
    bucket: z
      .string()
      .trim()
      .min(3, { message: BUCKET_INVALID })
      .max(63, { message: BUCKET_INVALID })
      .regex(/^[a-z0-9][a-z0-9.-]*[a-z0-9]$/, { message: BUCKET_INVALID })
      .refine((value) => !value.includes(".."), { message: BUCKET_INVALID }),
    publicBaseUrl: z.nullable(
      httpsUrlSchema(`${VALIDATION}.publicBaseUrlInvalid`, `${VALIDATION}.publicBaseUrlInvalid`),
    ),
  })
  .strict();
