/**
 * Explicit locale-key lookup for the backup/restore data-maintenance flow.
 *
 * Kept as a literal table (rather than building the key with a template
 * string like `settings.data.${operation}.confirm.title`) so every key is a
 * real string literal that tooling can find references to and rename, and
 * so the i18n completeness check doesn't need a manual `dynamic-keys.ts`
 * whitelist entry for this family of keys.
 */
export type DataMaintenanceOperation = "backup" | "restore";

export const LOCALE_KEYS_BY_DATA_MAINTENANCE_OPERATION: Record<
  DataMaintenanceOperation,
  { action: string; failed: string; confirmTitle: string; confirmDescription: string }
> = {
  backup: {
    action: "settings.data.backup.action",
    failed: "settings.data.backup.failed",
    confirmTitle: "settings.data.backup.confirm.title",
    confirmDescription: "settings.data.backup.confirm.description",
  },
  restore: {
    action: "settings.data.restore.action",
    failed: "settings.data.restore.failed",
    confirmTitle: "settings.data.restore.confirm.title",
    confirmDescription: "settings.data.restore.confirm.description",
  },
};
