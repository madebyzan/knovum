import { z } from "zod";

const settingsPaneIds = [
  "appearance",
  "aiModels",
  "desktopAgents",
  "executionRouting",
  "usage",
  "creation",
  "archivedSpaces",
  "about",
] as const;

export type SettingsPaneId = (typeof settingsPaneIds)[number];

const settingsPaneIdSchema = z.enum(settingsPaneIds);

export function resolveSettingsPaneId(value: unknown): SettingsPaneId | undefined {
  return settingsPaneIdSchema.safeParse(value).data;
}
