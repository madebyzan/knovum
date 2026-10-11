import { Boxes } from "@lucide/vue";

export const AI_SETTINGS_DEFINITION = {
  id: "aiModels",
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
    {
      id: "settings-ai-default-route",
      labelKey: "settings.routing.routes.defaultTarget.label",
      descriptionKey: "settings.routing.routes.defaultTarget.description",
    },
    {
      id: "settings-ai-routing",
      labelKey: "settings.routing.capabilities.title",
      descriptionKey: "settings.routing.defaults.description",
    },
  ],
} as const;
