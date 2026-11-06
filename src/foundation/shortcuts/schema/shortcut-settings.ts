import { z } from "zod";

export const shortcutSettingsSchema = z
  .object({
    custom: z.record(z.string(), z.string()).default({}),
  })
  .strict();

export type ShortcutSettings = z.infer<typeof shortcutSettingsSchema>;
