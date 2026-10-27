import { watchDebounced } from "@vueuse/core";
import { onScopeDispose, shallowRef, watch, type Ref } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { AgentExtensionSelection } from "../../../shared/contracts/generated/AgentExtensionSelection";
import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";
import { registerSpaceSessionParticipant } from "../workspace/session-participants";
import { createConversationDraftSession } from "./conversation-draft-session";

interface ConversationDraftPersistenceInput {
  spaceId: Ref<string | undefined>;
  conversationId: Ref<string>;
  prompt: Ref<string>;
  attachments: Ref<ConversationAttachment[]>;
  extensions: Ref<AgentExtensionSelection>;
}

export function useConversationDraftPersistence(input: ConversationDraftPersistenceInput) {
  const reacquireFileNames = shallowRef<string[]>([]);
  const error = shallowRef<string>();
  const session = createConversationDraftSession({
    spaceId: input.spaceId,
    selectedConversationId: input.conversationId,
    prompt: input.prompt,
    attachments: input.attachments,
    extensions: input.extensions,
    reacquireFileNames,
  });
  // A save reports on the draft it saved: one that settles after the composer has moved to
  // another Conversation says nothing about the draft now on screen.
  async function persist() {
    const owner = session.ownerEpoch();
    try {
      await session.remember();
      if (session.ownerEpoch() === owner) error.value = undefined;
    } catch (cause) {
      if (session.ownerEpoch() === owner) error.value = getErrorMessage(cause);
      throw cause;
    }
  }
  watch([input.spaceId, input.conversationId], () => (error.value = undefined), { flush: "sync" });
  const unregister = registerSpaceSessionParticipant({ phase: "persistence", commit: persist });
  onScopeDispose(() => {
    session.invalidate();
    unregister();
  });
  watchDebounced(
    [input.prompt, input.attachments, input.extensions, reacquireFileNames],
    () => void persist().catch(() => {}),
    { debounce: 300, deep: true },
  );
  return { session, error, reacquireFileNames };
}
