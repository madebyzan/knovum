import { Boxes } from "@lucide/vue";

export const AI_PROVIDERS_SETTINGS_DEFINITION = {
  id: "aiProviders",
  group: "intelligence",
  titleKey: "settings.ai.overview.title",
  descriptionKey: "settings.ai.overview.description",
  searchKeys: ["settings.ai.overview.description", "settings.ai.connections.title"],
  icon: Boxes,
  locations: [
    {
      id: "settings-ai-providers",
      labelKey: "settings.ai.connections.title",
      descriptionKey: "settings.ai.connections.description",
    },
  ],
} as const;
