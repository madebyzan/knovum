import { onBeforeUnmount, shallowRef, type Ref } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import {
  ApplicationLeaveCancelled,
  isApplicationLeaveCancelled,
} from "../../../../shared/contracts/application-leave";
import { registerSpaceSessionParticipant } from "../../workspace/session-participants";
import type { SpaceEditorTab } from "../../workspace/work-items";

interface VideoEditLeaveOptions {
  tab: SpaceEditorTab;
  dirty: Readonly<Ref<boolean>>;
  saving: Readonly<Ref<boolean>>;
  save: () => Promise<boolean>;
  close: () => Promise<void>;
  discard: () => void;
}

export function useVideoEditLeave(input: VideoEditLeaveOptions) {
  const closeDialogOpen = shallowRef(false);
  let resolveDecision: ((decision: "leave" | "discard" | "cancel") => void) | undefined;
  let pending: Promise<void> | undefined;
  let deferredDiscard = false;

  async function prepareLeave(): Promise<void> {
    if (pending) return pending;
    deferredDiscard = false;
    if (input.saving.value) {
      if (!(await input.save())) throw new Error("Video project save failed");
    }
    if (!input.dirty.value) return;
    closeDialogOpen.value = true;
    pending = new Promise<"leave" | "discard" | "cancel">((resolve) => (resolveDecision = resolve))
      .then((decision) => {
        if (decision === "cancel") throw new ApplicationLeaveCancelled();
        deferredDiscard = decision === "discard";
        return undefined;
      })
      .finally(() => {
        pending = undefined;
        resolveDecision = undefined;
      });
    return pending;
  }
  function finish(decision: "leave" | "discard" | "cancel") {
    closeDialogOpen.value = false;
    resolveDecision?.(decision);
  }
  const detach = registerSpaceSessionParticipant({
    editorOwner: { spaceId: input.tab.spaceId, tabId: input.tab.id, commitOnSwitch: false },
    commit: prepareLeave,
    finalize: () => {
      if (deferredDiscard) input.discard();
      deferredDiscard = false;
    },
  });
  onBeforeUnmount(() => {
    detach();
    if (pending) finish("cancel");
  });
  return {
    closeDialogOpen,
    cancel: () => finish("cancel"),
    discard: () => finish("discard"),
    requestClose: () => {
      void input.close().catch((error: unknown) => {
        if (!isApplicationLeaveCancelled(error)) toast.error(getErrorMessage(error));
      });
    },
    save: async () => {
      if ((await input.save()) && !input.dirty.value) finish("leave");
    },
  };
}
