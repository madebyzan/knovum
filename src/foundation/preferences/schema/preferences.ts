import { z } from "zod";

import {
  preferenceDensities,
  preferenceDefaults,
  preferenceLocales,
  preferencesSchemaVersion,
  preferenceTextSizes,
  preferenceThemes,
  workbenchWidthLimits,
} from "../../../shared/contracts/generated/contract-values";
import type { Preferences as PersistedPreferences } from "../../../shared/contracts/generated/Preferences";

export const PREFERENCES_SCHEMA_VERSION = preferencesSchemaVersion;

const preferencesSchema = z
  .object({
    theme: z.enum(preferenceThemes),
    density: z.enum(preferenceDensities),
    textSize: z.enum(preferenceTextSizes),
    locale: z.enum(preferenceLocales),
    backgroundTaskNotifications: z.boolean().default(preferenceDefaults.backgroundTaskNotifications),
    workbench: z
      .object({
        sidebarVisible: z.boolean().default(preferenceDefaults.workbench.sidebarVisible),
        sidebarWidth: z
          .number()
          .int()
          .min(workbenchWidthLimits.sidebar[0])
          .max(workbenchWidthLimits.sidebar[1])
          .default(preferenceDefaults.workbench.sidebarWidth),
        rightSidebarVisible: z.boolean().default(preferenceDefaults.workbench.rightSidebarVisible),
        rightSidebarWidth: z
          .number()
          .int()
          .min(workbenchWidthLimits.rightSidebar[0])
          .max(workbenchWidthLimits.rightSidebar[1])
          .default(preferenceDefaults.workbench.rightSidebarWidth),
      })
      .strict()
      .prefault({}),
  })
  .strict() satisfies z.ZodType<PersistedPreferences>;

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
export type WorkbenchPreferences = Preferences["workbench"];

export const defaultPreferences: Preferences = preferenceDefaults;

export function decodePreferences(raw: unknown): Preferences {
  const current = persistedSchema.safeParse(raw);
  if (current.success) return current.data.value;
  throw new Error("Persisted preferences use an unsupported or invalid schema.");
}
