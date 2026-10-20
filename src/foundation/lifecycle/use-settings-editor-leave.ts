import { onScopeDispose } from "vue";

import { useSettingsLeaveContext } from "./use-settings-leave-context";

export interface SettingsEditorLeaveOptions {
  pending: () => Promise<boolean> | undefined;
  dirty: () => boolean;
  confirmDiscard: () => Promise<boolean>;
  close: () => void;
}

/** Leaving a settings editor waits for its admitted write before deciding whether to discard. */
export function useSettingsEditorLeave(options: SettingsEditorLeaveOptions) {
  const participant = {
    async prepare() {
      const pending = options.pending();
      if (pending && !(await pending)) return false;
      return !options.dirty() || options.confirmDiscard();
    },
    finalize: options.close,
  };
  onScopeDispose(useSettingsLeaveContext().register(participant));

  return async function requestClose() {
    if (!(await participant.prepare())) return false;
    participant.finalize();
    return true;
  };
}
