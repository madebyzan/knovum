import { Route } from "@lucide/vue";

export const AI_ROUTING_SETTINGS_DEFINITION = {
  id: "aiRouting",
  group: "intelligence",
  titleKey: "settings.routing.title",
  descriptionKey: "settings.routing.description",
  searchKeys: ["settings.routing.description", "settings.routing.capabilities.title"],
  icon: Route,
  locations: [
    {
      id: "settings-ai-default-route",
      labelKey: "settings.routing.routes.defaultTarget.label",
      descriptionKey: "settings.routing.routes.defaultTarget.description",
    },
    {
      id: "settings-ai-routing",
      labelKey: "settings.routing.capabilities.title",
      descriptionKey: "settings.routing.capabilities.description",
    },
  ],
} as const;
