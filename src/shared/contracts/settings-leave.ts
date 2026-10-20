import type { InjectionKey } from "vue";

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
