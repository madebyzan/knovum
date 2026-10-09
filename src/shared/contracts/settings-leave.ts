import { inject, type InjectionKey } from "vue";

export interface SettingsLeaveParticipant {
  prepare(): Promise<boolean>;
  finalize?(): void;
}

export interface SettingsLeaveCoordinator {
  register(participant: SettingsLeaveParticipant): () => void;
  requestLeave(): Promise<void>;
}

export const settingsLeaveCoordinatorKey: InjectionKey<SettingsLeaveCoordinator> = Symbol(
  "settings-leave-coordinator",
);

export function useSettingsLeaveContext() {
  const coordinator = inject(settingsLeaveCoordinatorKey);
  if (!coordinator) {
    throw new Error("Settings leave participants must be composed within a settings leave coordinator.");
  }
  return coordinator;
}
