import type { Editor } from "@tiptap/vue-3";
import type { Ref } from "vue";

import { applySpaceNodeContent } from "../canvas/node";
import { isSpaceEditorNode } from "../canvas/node-capabilities";
import { useSpaceStore } from "../session/space-store";
import { useEditorAutosave } from "../workspace/use-editor-autosave";
import type { DocumentSession } from "./document-session-store";
import { persistSpaceTiptapContent } from "./tiptap-persistence";

export function useDocumentSave(
  session: DocumentSession,
  editor: Ref<Editor | undefined>,
  active: Ref<boolean>,
) {
  const store = useSpaceStore();
  const autosave = useEditorAutosave({
    tab: session.tab,
    active,
    dirty: session.dirty,
    saving: session.saving,
    error: session.error,
    edits: [session.state.revision, session.state.title],
    write: () => {
      const title = session.state.title.value;
      return persistSpaceTiptapContent({
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
      });
    },
    suspend: () => editor.value?.setEditable(false),
    resume: () => editor.value?.setEditable(active.value && !session.error.value),
  });
  return { persist: autosave.persist, suspended: autosave.suspended, saveStatus: autosave.status };
}
