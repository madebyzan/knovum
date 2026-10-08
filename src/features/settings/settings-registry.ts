import {
  Activity,
  Archive,
  AudioLines,
  Boxes,
  Info,
  MonitorCog,
  Puzzle,
  Terminal,
  WandSparkles,
} from "@lucide/vue";
import type { Component } from "vue";

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

export const settingsRegistry: readonly SettingsDefinition[] = [
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
    id: "aiModels",
    group: "intelligence",
    titleKey: "settings.ai.title",
    descriptionKey: "settings.ai.description",
    searchKeys: ["settings.ai.description", "settings.ai.connections"],
    icon: Boxes,
    locations: [
      {
        id: "settings-ai-providers",
        labelKey: "settings.ai.connections",
        descriptionKey: "settings.ai.connectionsDescription",
      },
      {
        id: "settings-ai-routing",
        labelKey: "settings.routing.defaultsTitle",
        descriptionKey: "settings.routing.defaultsDescription",
      },
    ],
  },
  {
    id: "voices",
    group: "intelligence",
    titleKey: "spaces.speech.library.title",
    descriptionKey: "spaces.speech.library.description",
    searchKeys: ["spaces.speech.reference.title", "spaces.speech.voice.clone", "spaces.speech.voice.design"],
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
    searchKeys: [
      "extensions.overview.plugins",
      "extensions.overview.skills",
      "extensions.overview.connectors",
    ],
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
      {
        id: "settings-image-layers",
        labelKey: "settings.creation.imageLayers.title",
        descriptionKey: "settings.creation.imageLayers.description",
      },
    ],
  },
  {
    id: "archivedSpaces",
    group: "data",
    titleKey: "settings.general.archived.title",
    descriptionKey: "settings.general.archived.description",
    searchKeys: ["settings.general.archived.description"],
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

export const settingsGroupTitleKeys: Record<SettingsGroupId, string> = {
  general: "settings.general.groups.general",
  intelligence: "settings.general.groups.intelligence",
  data: "settings.general.groups.data",
  information: "settings.general.groups.information",
};

export function settingsDefinition(id: SettingsPaneId) {
  return settingsRegistry.find((definition) => definition.id === id)!;
}
