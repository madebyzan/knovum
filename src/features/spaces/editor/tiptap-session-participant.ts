import type { Editor } from "@tiptap/vue-3";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { registerSpaceSessionParticipant } from "../canvas/editor-session";

interface TiptapSessionParticipantOptions {
  cancelAutoSave(): void;
  editor(): Editor | undefined;
  persist(): Promise<void>;
}

export function registerTiptapSessionParticipant(options: TiptapSessionParticipantOptions) {
  return registerSpaceSessionParticipant({
    async commit() {
      try {
        await persistTiptapSession(options.persist);
      } catch (error) {
        toast.error(getErrorMessage(error));
        throw error;
      }
    },
    suspend() {
      options.cancelAutoSave();
      options.editor()?.setEditable(false);
    },
    resume: () => options.editor()?.setEditable(true),
  });
}

export async function persistTiptapSession(persist: () => Promise<void>, afterPersist = () => {}) {
  await persist();
  afterPersist();
}
