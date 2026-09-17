import { isString } from "es-toolkit";

const settingsPaneIds = [
  "appearance",
  "startup",
  "aiModels",
  "desktopAgents",
  "executionRouting",
  "creation",
  "archivedSpaces",
  "about",
] as const;

export type SettingsPaneId = (typeof settingsPaneIds)[number];

export function resolveSettingsPaneId(value: unknown): SettingsPaneId | undefined {
  if (!isString(value)) return undefined;
  return settingsPaneIds.includes(value as SettingsPaneId) ? (value as SettingsPaneId) : undefined;
}
