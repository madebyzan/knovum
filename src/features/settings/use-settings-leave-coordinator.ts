import { onUnmounted, provide } from "vue";
import { onBeforeRouteLeave, onBeforeRouteUpdate } from "vue-router";

import { registerWindowCloseCommit } from "../../foundation/lifecycle/window-close-coordinator";
import {
  ApplicationLeaveCancelled,
  isApplicationLeaveCancelled,
} from "../../shared/contracts/application-leave";
import {
  settingsLeaveCoordinatorKey,
  type SettingsLeaveParticipant,
} from "../../shared/contracts/settings-leave";

export function useSettingsLeaveCoordinator() {
  const participants = new Set<SettingsLeaveParticipant>();
  let pending: Promise<void> | undefined;
  async function decideLeave() {
    const prepared = [...participants];
    for (const participant of prepared) {
      // oxlint-disable-next-line no-await-in-loop -- leave prompts must resolve one at a time.
      if (!(await participant.prepare())) throw new ApplicationLeaveCancelled();
    }
    for (const participant of prepared) participant.finalize?.();
  }
  function requestLeave() {
    pending ??= decideLeave().finally(() => (pending = undefined));
    return pending;
  }
  provide(settingsLeaveCoordinatorKey, {
    register(participant) {
      participants.add(participant);
      return () => participants.delete(participant);
    },
    requestLeave,
  });
  async function allowRouteLeave() {
    try {
      await requestLeave();
      return true;
    } catch (error) {
      if (isApplicationLeaveCancelled(error)) return false;
      throw error;
    }
  }

  const unregisterWindowCloseCommit = registerWindowCloseCommit(requestLeave);
  onUnmounted(unregisterWindowCloseCommit);
  onBeforeRouteLeave(allowRouteLeave);
  onBeforeRouteUpdate((to, from) => (to.query.pane === from.query.pane ? true : allowRouteLeave()));
}
