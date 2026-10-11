import type { Editor } from "@tiptap/vue-3";
import { useDebounceFn } from "@vueuse/core";
import { computed, onBeforeUnmount, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { applySpaceNodeContent } from "../canvas/node";
import { isSpaceEditorNode } from "../canvas/node-capabilities";
import { useSpaceStore } from "../session/space-store";
import { registerSpaceSessionParticipant } from "../workspace/session-participants";
import type { DocumentSession } from "./document-session-store";
import { persistSpaceTiptapContent } from "./tiptap-persistence";

export function useDocumentSave(
  session: DocumentSession,
  editor: Ref<Editor | undefined>,
  active: Ref<boolean>,
) {
  const store = useSpaceStore();
  const { t } = useI18n();
  const failed = shallowRef(false);
  const suspended = shallowRef(false);
  let pending: Promise<void> | undefined;
  function persist(): Promise<void> {
    if (pending) return pending.then(() => (session.dirty.value ? persist() : undefined));
    if (!session.dirty.value) return Promise.resolve();
    if (session.error.value) return Promise.reject(new Error(t(`spaces.editor.tabs.${session.error.value}`)));
    session.saving.value = true;
    failed.value = false;
    const title = session.state.title.value;
    pending = persistSpaceTiptapContent({
      node: session.node.value,
      state: session.state,
      editor: editor.value,
      save: (change) => store.saveNodeContentForSpace(session.tab.spaceId, change),
      onSaved: (payload) => {
        const node = applySpaceNodeContent(session.node.value, payload);
        if (!isSpaceEditorNode(node)) throw new Error("Saved document has an invalid type");
        session.node.value = node;
        if (session.state.title.value === title) session.state.title.value = payload.title;
      },
    })
      .catch((error: unknown) => {
        failed.value = true;
        throw error;
      })
      .finally(() => {
        session.saving.value = false;
        pending = undefined;
      });
    return pending;
  }
  const autoSave = useDebounceFn(() => {
    void persist().catch((error: unknown) => toast.error(getErrorMessage(error)));
  }, 1000);
  watch([session.state.revision, session.state.title], () => {
    if (active.value) autoSave();
  });
  watch(active, (visible) => {
    if (!visible) autoSave.cancel();
    else if (session.dirty.value) autoSave();
  });
  const detach = registerSpaceSessionParticipant({
    editorOwner: { spaceId: session.tab.spaceId, tabId: session.tab.id, commitOnSwitch: true },
    commit: async () => {
      do {
        // oxlint-disable-next-line eslint/no-await-in-loop -- Drain edits admitted during the preceding write before closing.
        await persist();
      } while (session.dirty.value);
    },
    suspend: () => {
      suspended.value = true;
      autoSave.cancel();
      editor.value?.setEditable(false);
    },
    resume: () => {
      suspended.value = false;
      editor.value?.setEditable(active.value && !session.error.value);
    },
  });
  onBeforeUnmount(() => {
    detach();
    autoSave.cancel();
  });
  return {
    persist,
    suspended,
    saveStatus: computed(() =>
      session.saving.value ? "saving" : session.dirty.value || failed.value ? "unsaved" : "saved",
    ),
  };
}
