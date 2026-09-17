import type { Editor } from "@tiptap/vue-3";
import { toast } from "vue-sonner";

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
        await options.persist();
      } catch (error) {
        toast.error(String(error));
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
