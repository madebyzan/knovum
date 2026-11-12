import { z } from "zod";

const SETTINGS_PANE_IDS = [
  "appearance",
  "notifications",
  "shortcuts",
  "aiRouting",
  "aiProviders",
  "localModels",
  "voices",
  "desktopAgents",
  "extensions",
  "usage",
  "data",
  "objectStorage",
  "archivedSpaces",
  "about",
] as const;

export type SettingsPaneId = (typeof SETTINGS_PANE_IDS)[number];

const settingsPaneIdSchema = z.enum(SETTINGS_PANE_IDS);

export function resolveSettingsPaneId(value: unknown): SettingsPaneId | undefined {
  return settingsPaneIdSchema.safeParse(value).data;
}
