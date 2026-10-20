import { inject } from "vue";

import { settingsLeaveCoordinatorKey } from "../../shared/contracts/settings-leave";

export function useSettingsLeaveContext() {
  const coordinator = inject(settingsLeaveCoordinatorKey);
  if (!coordinator) {
    throw new Error("Settings leave participants must be composed within a settings leave coordinator.");
  }
  return coordinator;
}
