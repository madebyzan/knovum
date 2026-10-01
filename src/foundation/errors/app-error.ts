import { z } from "zod";

import type { AppError } from "../../shared/contracts/generated/AppError";

const appErrorSchema: z.ZodType<AppError> = z
  .object({ code: z.string().regex(/^[a-z][a-z0-9]*(?:_[a-z0-9]+)*$/), message: z.string() })
  .strict();

export function decodeAppError(reason: unknown): AppError | null {
  const parsed = appErrorSchema.safeParse(reason instanceof Error ? reason.cause : reason);
  return parsed.success ? parsed.data : null;
}

export function normalizeError(reason: unknown): Error {
  if (reason instanceof Error) return reason;
  const payload = decodeAppError(reason);
  if (payload) return new Error(payload.message, { cause: payload });
  return new Error(typeof reason === "string" ? reason : "Unexpected error", { cause: reason });
}

export function createAppError(code: AppError["code"], message: string): Error {
  return new Error(message, { cause: { code, message } satisfies AppError });
}
