import { onScopeDispose } from "vue";

import {
  type SettingsLeaveParticipant,
  useSettingsLeaveContext,
} from "../../../shared/contracts/settings-leave";

export function useSettingsLeaveDecision(participant: SettingsLeaveParticipant) {
  const unregister = useSettingsLeaveContext().register(participant);
  onScopeDispose(unregister);
}
