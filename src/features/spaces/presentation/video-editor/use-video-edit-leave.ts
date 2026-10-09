import { onMounted, onUnmounted, shallowRef, type Ref } from "vue";

import {
  ApplicationLeaveCancelled,
  isApplicationLeaveCancelled,
} from "../../../../shared/contracts/application-leave";
import { registerSpaceSessionParticipant } from "../../workspace/session-participants";

interface VideoEditLeaveOptions {
  dirty: Readonly<Ref<boolean>>;
  saving: Readonly<Ref<boolean>>;
  save: () => Promise<boolean>;
  discard: () => void;
  close: () => void;
}

export function useVideoEditLeave(input: VideoEditLeaveOptions) {
  const closeDialogOpen = shallowRef(false);
  let resolveDecision: ((decision: "leave" | "discard" | "cancel") => void) | undefined;
  let pending: Promise<void> | undefined;
  let closeAfterDecision = false;
  let deferredDiscard = false;
  let deferredClose = false;

  async function prepareLeave(closeAfter = false): Promise<void> {
    closeAfterDecision ||= closeAfter;
    if (pending) return pending;
    deferredDiscard = false;
    deferredClose = false;
    if (input.saving.value) {
      await input.save();
      return prepareLeave(closeAfter);
    }
    if (!input.dirty.value) {
      deferredClose = closeAfterDecision;
      closeAfterDecision = false;
      return;
    }
    closeDialogOpen.value = true;
    pending = new Promise<"leave" | "discard" | "cancel">((resolve) => (resolveDecision = resolve))
      .then((decision) => {
        if (decision === "cancel") throw new ApplicationLeaveCancelled();
        deferredDiscard = decision === "discard";
        deferredClose = closeAfterDecision;
        return undefined;
      })
      .finally(() => {
        closeAfterDecision = false;
        pending = undefined;
        resolveDecision = undefined;
      });
    return pending;
  }

  function finalizeLeave() {
    if (deferredDiscard) input.discard();
    else if (deferredClose) input.close();
    deferredDiscard = false;
    deferredClose = false;
  }

  async function requestLeave(closeAfter = false) {
    await prepareLeave(closeAfter);
    finalizeLeave();
  }

  function finish(decision: "leave" | "discard" | "cancel") {
    closeDialogOpen.value = false;
    if (decision === "cancel") closeAfterDecision = false;
    resolveDecision?.(decision);
  }

  function requestClose() {
    void requestLeave(true).catch((error) => {
      if (!isApplicationLeaveCancelled(error)) throw error;
    });
  }

  let unregister: (() => void) | undefined;
  onMounted(() => {
    unregister = registerSpaceSessionParticipant({ commit: prepareLeave, finalize: finalizeLeave });
  });
  onUnmounted(() => {
    unregister?.();
    if (pending) finish("cancel");
  });

  return {
    cancel: () => finish("cancel"),
    closeDialogOpen,
    discard: () => finish("discard"),
    requestClose,
    save: async () => {
      if ((await input.save()) && !input.dirty.value) finish("leave");
    },
  };
}
