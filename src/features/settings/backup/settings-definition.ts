import { DatabaseBackup } from "@lucide/vue";

export const backupSettingsDefinition = {
  id: "data",
  group: "data",
  titleKey: "settings.data.title",
  descriptionKey: "settings.data.description",
  searchKeys: ["settings.data.backup.title", "settings.data.restore.title"],
  icon: DatabaseBackup,
  locations: [
    {
      id: "settings-data-backup",
      labelKey: "settings.data.backup.title",
      descriptionKey: "settings.data.backup.description",
    },
    {
      id: "settings-data-restore",
      labelKey: "settings.data.restore.title",
      descriptionKey: "settings.data.restore.description",
    },
  ],
} as const;
