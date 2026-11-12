import { Cloud } from "@lucide/vue";

export const OBJECT_STORAGE_SETTINGS_DEFINITION = {
  id: "objectStorage",
  group: "data",
  titleKey: "settings.objectStorage.title",
  descriptionKey: "settings.objectStorage.description",
  searchKeys: ["settings.objectStorage.connection.title", "settings.objectStorage.credentials.title"],
  icon: Cloud,
  locations: [
    {
      id: "settings-object-storage-connection",
      labelKey: "settings.objectStorage.connection.title",
      descriptionKey: "settings.objectStorage.connection.description",
    },
    {
      id: "settings-object-storage-credentials",
      labelKey: "settings.objectStorage.credentials.title",
      descriptionKey: "settings.objectStorage.credentials.description",
    },
  ],
} as const;
