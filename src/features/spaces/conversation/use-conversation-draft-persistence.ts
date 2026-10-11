import { watchDebounced } from "@vueuse/core";
import { onScopeDispose, shallowRef, type Ref } from "vue";

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
  async function persist() {
    try {
      await session.remember();
      error.value = undefined;
    } catch (cause) {
      error.value = getErrorMessage(cause);
      throw cause;
    }
  }
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
