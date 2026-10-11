import type { Editor } from "@tiptap/vue-3";

import { registerSpaceSessionParticipant } from "../workspace/session-participants";

interface TiptapSessionParticipantOptions {
  cancelAutoSave(): void;
  editor(): Editor | undefined;
  persist(): Promise<void>;
}

export function registerTiptapSessionParticipant(options: TiptapSessionParticipantOptions) {
  return registerSpaceSessionParticipant({
    commit: options.persist,
    suspend() {
      options.cancelAutoSave();
      options.editor()?.setEditable(false);
    },
    resume: () => options.editor()?.setEditable(true),
  });
}
