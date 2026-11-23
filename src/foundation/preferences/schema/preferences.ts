import { z } from "zod";

import {
  canvasBackdrops,
  canvasBackgroundPatterns,
  preferenceDensities,
  preferenceDefaults,
  preferenceLocales,
  preferencesSchemaVersion,
  preferenceTextSizes,
  preferenceThemes,
  workbenchWidthLimits,
} from "../../../shared/contracts/generated/contract-values";
import type { Preferences as PersistedPreferences } from "../../../shared/contracts/generated/Preferences";

const preferencesSchema = z
  .object({
    theme: z.enum(preferenceThemes),
    density: z.enum(preferenceDensities),
    textSize: z.enum(preferenceTextSizes),
    locale: z.enum(preferenceLocales),
    backgroundTaskNotifications: z.boolean().default(preferenceDefaults.backgroundTaskNotifications),
    restorePreviousWorkspace: z.boolean().default(preferenceDefaults.restorePreviousWorkspace),
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
    canvas: z
      .object({
        backgroundPattern: z
          .enum(canvasBackgroundPatterns)
          .default(preferenceDefaults.canvas.backgroundPattern),
        backdrop: z.enum(canvasBackdrops).default(preferenceDefaults.canvas.backdrop),
      })
      .strict()
      .prefault({}),
  })
  .strict() satisfies z.ZodType<PersistedPreferences>;

const persistedSchema = z
  .object({
    schemaVersion: z.literal(preferencesSchemaVersion),
    value: preferencesSchema,
  })
  .strict();

export type Preferences = z.infer<typeof preferencesSchema>;
export type ThemePreference = Preferences["theme"];
export type DensityPreference = Preferences["density"];
export type TextSizePreference = Preferences["textSize"];
export type LocalePreference = Preferences["locale"];
export type WorkbenchPreferences = Preferences["workbench"];
type CanvasPreferences = Preferences["canvas"];
export type CanvasBackgroundPattern = CanvasPreferences["backgroundPattern"];
export type CanvasBackdrop = CanvasPreferences["backdrop"];

export function decodePreferences(raw: unknown): Preferences {
  const current = persistedSchema.safeParse(raw);
  if (current.success) return current.data.value;
  throw new Error("Persisted preferences use an unsupported or invalid schema.");
}
