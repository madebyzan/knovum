import { z } from "zod";

export const packageTimeSchema = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);
export const packageIdSchema = z.string().min(1).max(256);
