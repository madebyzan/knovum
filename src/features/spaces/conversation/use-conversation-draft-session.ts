import type { Ref } from "vue";

import { useConversationDraftStore } from "./conversation-draft-store";
import type { StartAssistantTaskInput } from "./conversation-ipc";

type Attachment = NonNullable<StartAssistantTaskInput["attachments"]>[number];

function sameAttachments(left: readonly Attachment[], right: readonly Attachment[]) {
  return (
    left.length === right.length &&
    left.every((item) =>
      right.some(
        (attachment) =>
          attachment.nodeId === item.nodeId &&
          attachment.sourceRevision === item.sourceRevision &&
          attachment.role === item.role,
      ),
    )
  );
}

export function useConversationDraftSession(input: {
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  prompt: Ref<string>;
  attachments: Ref<Attachment[]>;
}) {
  const drafts = useConversationDraftStore();

  function remember(id = input.selectedConversationId.value, currentSpaceId = input.spaceId.value) {
    if (!currentSpaceId) return;
    drafts.save(currentSpaceId, id, {
      prompt: input.prompt.value,
      attachments: input.attachments.value,
    });
  }

  function restore(id: string) {
    const draft = input.spaceId.value ? drafts.read(input.spaceId.value, id) : undefined;
    input.prompt.value = draft?.prompt ?? "";
    input.attachments.value = draft?.attachments ?? [];
    return Boolean(draft);
  }

  function move(fromId: string, toId: string) {
    if (!input.spaceId.value) return;
    drafts.move(input.spaceId.value, fromId, toId, {
      prompt: input.prompt.value,
      attachments: input.attachments.value,
    });
  }

  function clearSubmitted(request: StartAssistantTaskInput) {
    const conversationId = request.conversationId ?? "";
    const stored = drafts.read(request.spaceId, conversationId);
    const sent = request.attachments ?? [];
    if (stored?.prompt.trim() === request.prompt && sameAttachments(stored.attachments, sent)) {
      drafts.remove(request.spaceId, conversationId);
    }
    if (input.spaceId.value !== request.spaceId || input.selectedConversationId.value !== conversationId)
      return;
    if (input.prompt.value.trim() === request.prompt) input.prompt.value = "";
    if (sameAttachments(input.attachments.value, sent)) input.attachments.value = [];
  }

  return { remember, restore, move, clearSubmitted };
}
