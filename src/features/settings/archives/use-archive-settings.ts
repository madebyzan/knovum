import { inject } from "vue";

import { archiveSettingsKey } from "../../../shared/contracts/archive-settings";

export function useArchiveSettings() {
  const archive = inject(archiveSettingsKey);
  if (!archive) throw new Error("Archive settings contract is required");
  return archive;
}
