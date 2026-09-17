import { z } from "zod";

export const PREFERENCES_SCHEMA_VERSION = 1;

const preferencesSchema = z
  .object({
    theme: z.enum(["system", "light", "dark"]),
    density: z.enum(["compact", "comfortable"]),
    textSize: z.enum(["small", "default", "large"]),
    locale: z.enum(["system", "en", "zh-CN"]),
  })
  .strict();

const persistedSchema = z
  .object({
    schemaVersion: z.literal(PREFERENCES_SCHEMA_VERSION),
    value: preferencesSchema,
  })
  .strict();

export type Preferences = z.infer<typeof preferencesSchema>;
export type ThemePreference = Preferences["theme"];
export type DensityPreference = Preferences["density"];
export type TextSizePreference = Preferences["textSize"];
export type LocalePreference = Preferences["locale"];

export const defaultPreferences: Preferences = {
  theme: "system",
  density: "compact",
  textSize: "default",
  locale: "system",
};

export function decodePreferences(raw: unknown): Preferences {
  const current = persistedSchema.safeParse(raw);
  if (current.success) return current.data.value;
  throw new Error("Persisted preferences use an unsupported or invalid schema.");
}
