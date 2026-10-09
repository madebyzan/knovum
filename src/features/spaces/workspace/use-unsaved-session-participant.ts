import { onMounted, onUnmounted, shallowRef, type Ref } from "vue";

import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { registerSpaceSessionParticipant } from "./session-participants";

interface UnsavedSessionParticipantOptions {
  dirty: Readonly<Ref<boolean>>;
  save: () => Promise<boolean>;
  discard?: () => void;
  leaveClean?: () => void;
  beforeLeave?: () => Promise<void>;
}

export function useUnsavedSessionParticipant(options: UnsavedSessionParticipantOptions) {
  const dialogOpen = shallowRef(false);
  const saving = shallowRef(false);
  let resolveDecision: ((decision: "leave" | "discard" | "cancel") => void) | undefined;
  let deferredCleanup: (() => void) | undefined;
  let pending: Promise<void> | undefined;

  async function prepareLeave() {
    deferredCleanup = undefined;
    await options.beforeLeave?.();
    if (!options.dirty.value) {
      deferredCleanup = options.leaveClean;
      return;
    }
    if (pending) return pending;
    dialogOpen.value = true;
    pending = new Promise<"leave" | "discard" | "cancel">((resolve) => (resolveDecision = resolve)).then(
      (decision) => {
        if (decision === "cancel") throw new ApplicationLeaveCancelled();
        if (decision === "discard") deferredCleanup = options.discard;
        return undefined;
      },
    );
    const clearPending = () => {
      pending = undefined;
      resolveDecision = undefined;
    };
    void pending.then(clearPending, clearPending);
    return pending;
  }

  function finalizeLeave() {
    const cleanup = deferredCleanup;
    deferredCleanup = undefined;
    cleanup?.();
  }

  async function requestLeave() {
    await prepareLeave();
    finalizeLeave();
  }

  function finish(decision: "leave" | "discard" | "cancel") {
    dialogOpen.value = false;
    resolveDecision?.(decision);
  }

  async function save() {
    if (saving.value) return;
    saving.value = true;
    try {
      if (await options.save()) finish("leave");
    } finally {
      saving.value = false;
    }
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
    dialogOpen,
    saving,
    requestLeave,
    cancel: () => finish("cancel"),
    discard: () => finish("discard"),
    save,
  };
}
