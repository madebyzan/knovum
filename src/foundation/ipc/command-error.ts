import { z } from "zod";

import type { CommandError } from "../../shared/contracts/generated/CommandError";

const commandErrorSchema: z.ZodType<CommandError> = z
  .object({ code: z.string().min(1), message: z.string() })
  .strict();
export function decodeCommandError(reason: unknown): CommandError | null {
  const parsed = commandErrorSchema.safeParse(reason instanceof Error ? reason.cause : reason);
  return parsed.success ? parsed.data : null;
}

export function normalizeCommandError(reason: unknown): Error {
  if (reason instanceof Error) return reason;
  const payload = decodeCommandError(reason);
  if (payload) return new Error(payload.message, { cause: payload });
  // Tauri itself and official plugins can reject before the application command runs.
  return new Error(typeof reason === "string" ? reason : "Desktop command failed", { cause: reason });
}
