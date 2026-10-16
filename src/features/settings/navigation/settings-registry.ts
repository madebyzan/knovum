import {
  Activity,
  Archive,
  AudioLines,
  Bell,
  Info,
  HardDrive,
  MonitorCog,
  Puzzle,
  Terminal,
  WandSparkles,
} from "@lucide/vue";
import type { Component } from "vue";

import { AI_ROUTING_SETTINGS_DEFINITION } from "../ai/routing-settings-definition";
import { AI_PROVIDERS_SETTINGS_DEFINITION } from "../ai/settings-definition";
import { BACKUP_SETTINGS_DEFINITION } from "../backup/settings-definition";
import type { SettingsPaneId } from "./settings-pane";

export type SettingsGroupId = "general" | "intelligence" | "data" | "information";

export interface SettingsDefinition {
  id: SettingsPaneId;
  group: SettingsGroupId;
  titleKey: string;
  descriptionKey: string;
  searchKeys: readonly string[];
  icon: Component;
  locations?: readonly { id: string; labelKey: string; descriptionKey: string }[];
}

export const SETTINGS_REGISTRY: readonly SettingsDefinition[] = [
  {
    id: "appearance",
    group: "general",
    titleKey: "settings.appearance.title",
    descriptionKey: "settings.appearance.description",
    searchKeys: ["settings.appearance.description"],
    icon: MonitorCog,
    locations: [
      {
        id: "settings-appearance-theme",
        labelKey: "settings.appearance.theme.label",
        descriptionKey: "settings.appearance.theme.description",
      },
      {
        id: "settings-appearance-text-size",
        labelKey: "settings.appearance.textSize.label",
        descriptionKey: "settings.appearance.textSize.description",
      },
      {
        id: "settings-appearance-density",
        labelKey: "settings.appearance.density.label",
        descriptionKey: "settings.appearance.density.description",
      },
      {
        id: "settings-appearance-language",
        labelKey: "settings.appearance.language.label",
        descriptionKey: "settings.appearance.language.description",
      },
    ],
  },
  {
    id: "notifications",
    group: "general",
    titleKey: "settings.notifications.title",
    descriptionKey: "settings.notifications.description",
    searchKeys: ["settings.notifications.backgroundTasks.label"],
    icon: Bell,
    locations: [
      {
        id: "settings-background-task-notifications",
        labelKey: "settings.notifications.backgroundTasks.label",
        descriptionKey: "settings.notifications.backgroundTasks.description",
      },
    ],
  },
  AI_ROUTING_SETTINGS_DEFINITION,
  AI_PROVIDERS_SETTINGS_DEFINITION,
  {
    id: "localModels",
    group: "intelligence",
    titleKey: "settings.ai.localModels.title",
    descriptionKey: "settings.ai.localModels.description",
    searchKeys: ["settings.ai.localModels.description"],
    icon: HardDrive,
  },
  {
    id: "voices",
    group: "intelligence",
    titleKey: "spaces.speech.library.title",
    descriptionKey: "spaces.speech.library.description",
    searchKeys: [
      "spaces.speech.reference.title",
      "spaces.speech.voice.clone.fromReference",
      "spaces.speech.voice.design.action",
    ],
    icon: AudioLines,
  },
  {
    id: "desktopAgents",
    group: "intelligence",
    titleKey: "settings.agents.title",
    descriptionKey: "settings.agents.description",
    searchKeys: ["settings.agents.description"],
    icon: Terminal,
    locations: [
      {
        id: "settings-desktop-agents",
        labelKey: "settings.agents.title",
        descriptionKey: "settings.agents.description",
      },
    ],
  },
  {
    id: "extensions",
    group: "intelligence",
    titleKey: "extensions.overview.title",
    descriptionKey: "extensions.overview.description",
    searchKeys: ["extensions.overview.plugins", "extensions.overview.skills", "extensions.overview.mcp"],
    icon: Puzzle,
  },
  {
    id: "usage",
    group: "intelligence",
    titleKey: "settings.usage.title",
    descriptionKey: "settings.usage.description",
    searchKeys: ["settings.usage.activity.title", "settings.usage.tokens.total"],
    icon: Activity,
  },
  {
    id: "creation",
    group: "intelligence",
    titleKey: "settings.creation.title",
    descriptionKey: "settings.creation.description",
    searchKeys: ["settings.creation.description", "settings.creation.presets.title"],
    icon: WandSparkles,
    locations: [
      {
        id: "settings-generation-presets",
        labelKey: "settings.creation.presets.title",
        descriptionKey: "settings.creation.presets.description",
      },
    ],
  },
  BACKUP_SETTINGS_DEFINITION,
  {
    id: "archivedSpaces",
    group: "data",
    titleKey: "settings.archivedSpaces.title",
    descriptionKey: "settings.archivedSpaces.description",
    searchKeys: ["settings.archivedSpaces.description"],
    icon: Archive,
  },
  {
    id: "about",
    group: "information",
    titleKey: "settings.general.about.title",
    descriptionKey: "settings.general.about.description",
    searchKeys: ["settings.general.about.description"],
    icon: Info,
  },
] as const;

export const SETTINGS_GROUP_TITLE_KEYS: Record<SettingsGroupId, string> = {
  general: "settings.general.groups.general",
  intelligence: "settings.general.groups.intelligence",
  data: "settings.general.groups.data",
  information: "settings.general.groups.information",
};

export function getSettingsDefinition(paneId: SettingsPaneId) {
  return SETTINGS_REGISTRY.find((definition) => definition.id === paneId)!;
}
