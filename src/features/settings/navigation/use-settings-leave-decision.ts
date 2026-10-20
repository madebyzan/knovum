import { onScopeDispose } from "vue";

import { useSettingsLeaveContext } from "../../../foundation/lifecycle/use-settings-leave-context";
import type { SettingsLeaveParticipant } from "../../../shared/contracts/settings-leave";

export function useSettingsLeaveDecision(participant: SettingsLeaveParticipant) {
  const unregister = useSettingsLeaveContext().register(participant);
  onScopeDispose(unregister);
}
