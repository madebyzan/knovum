import { Archive, Boxes, Info, MonitorCog, Power, Route, Terminal, WandSparkles } from "@lucide/vue";
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
        labelKey: "settings.appearance.theme",
        descriptionKey: "settings.appearance.themeDescription",
      },
      {
        id: "settings-appearance-text-size",
        labelKey: "settings.appearance.textSize",
        descriptionKey: "settings.appearance.textSizeDescription",
      },
      {
        id: "settings-appearance-density",
        labelKey: "settings.appearance.density",
        descriptionKey: "settings.appearance.densityDescription",
      },
      {
        id: "settings-appearance-language",
        labelKey: "settings.appearance.language",
        descriptionKey: "settings.appearance.languageDescription",
      },
    ],
  },
  {
    id: "startup",
    group: "general",
    titleKey: "settings.startup.title",
    descriptionKey: "settings.startup.description",
    searchKeys: ["settings.startup.description"],
    icon: Power,
  },
  {
    id: "aiModels",
    group: "intelligence",
    titleKey: "settings.ai.title",
    descriptionKey: "settings.ai.description",
    searchKeys: ["settings.ai.description", "settings.ai.connections", "settings.runtime.title"],
    icon: Boxes,
    locations: [
      {
        id: "settings-ai-providers",
        labelKey: "settings.ai.connections",
        descriptionKey: "settings.ai.connectionsDescription",
      },
      {
        id: "settings-local-models",
        labelKey: "settings.runtime.title",
        descriptionKey: "settings.runtime.description",
      },
    ],
  },
  {
    id: "desktopAgents",
    group: "intelligence",
    titleKey: "settings.ai.desktopAgentsTitle",
    descriptionKey: "settings.ai.desktopAgentsDescription",
    searchKeys: ["settings.ai.desktopAgentsDescription"],
    icon: Terminal,
    locations: [
      {
        id: "settings-desktop-agents",
        labelKey: "settings.ai.desktopAgentsTitle",
        descriptionKey: "settings.ai.desktopAgentsDescription",
      },
    ],
  },
  {
    id: "executionRouting",
    group: "intelligence",
    titleKey: "settings.ai.executionRoutingTitle",
    descriptionKey: "settings.ai.executionRoutingDescription",
    searchKeys: ["settings.ai.defaultRoutes", "settings.ai.defaultRoutesDescription"],
    icon: Route,
    locations: [
      {
        id: "settings-ai-routing",
        labelKey: "settings.ai.defaultRoutes",
        descriptionKey: "settings.ai.defaultRoutesDescription",
      },
    ],
  },
  {
    id: "creation",
    group: "intelligence",
    titleKey: "settings.creation.title",
    descriptionKey: "settings.creation.description",
    searchKeys: ["settings.creation.description", "settings.creation.presets", "settings.creation.workflows"],
    icon: WandSparkles,
    locations: [
      {
        id: "settings-generation-presets",
        labelKey: "settings.creation.presets",
        descriptionKey: "settings.creation.presetsDescription",
      },
      {
        id: "settings-creation-workflows",
        labelKey: "settings.creation.workflows",
        descriptionKey: "settings.creation.workflowsDescription",
      },
    ],
  },
  {
    id: "archivedSpaces",
    group: "data",
    titleKey: "settings.general.archivedSpaces",
    descriptionKey: "settings.general.archivedSpacesDescription",
    searchKeys: ["settings.general.archivedSpacesDescription"],
    icon: Archive,
  },
  {
    id: "about",
    group: "information",
    titleKey: "settings.general.about",
    descriptionKey: "settings.general.aboutDescription",
    searchKeys: ["settings.general.aboutDescription"],
    icon: Info,
  },
] as const;

export const settingsGroupTitleKeys: Record<SettingsGroupId, string> = {
  general: "settings.general.preferences",
  intelligence: "settings.ai.knowledgeSpace",
  data: "settings.general.data",
  information: "settings.general.information",
};

export function settingsDefinition(id: SettingsPaneId) {
  return settingsRegistry.find((definition) => definition.id === id)!;
}
