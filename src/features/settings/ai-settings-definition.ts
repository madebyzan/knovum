import { Boxes } from "@lucide/vue";

export const aiSettingsDefinition = {
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
      id: "settings-ai-default-route",
      labelKey: "settings.routing.routes.defaultTarget.label",
      descriptionKey: "settings.routing.routes.defaultTarget.description",
    },
    {
      id: "settings-ai-routing",
      labelKey: "settings.routing.capabilitiesTitle",
      descriptionKey: "settings.routing.defaultsDescription",
    },
  ],
} as const;
