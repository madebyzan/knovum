import type { Ref } from "vue";

import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";
import type { SubmitConversationTaskRequest } from "../contracts/generated/SubmitConversationTaskRequest";
import { useConversationDraftStore } from "./conversation-draft-store";

function sameAttachments(left: readonly ConversationAttachment[], right: readonly ConversationAttachment[]) {
  return (
    left.length === right.length &&
    left.every((item) =>
      right.some(
        (attachment) =>
          attachment.kind === item.kind &&
          (attachment.kind === "node" && item.kind === "node"
            ? attachment.nodeId === item.nodeId
            : attachment.kind === "file" && item.kind === "file" && attachment.path === item.path) &&
          attachment.sourceRevision === item.sourceRevision &&
          (attachment.kind === "file" || item.kind === "file" || attachment.role === item.role),
      ),
    )
  );
}

interface ConversationDraftSessionInput {
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  prompt: Ref<string>;
  attachments: Ref<ConversationAttachment[]>;
}

export function useConversationDraftSession(input: ConversationDraftSessionInput) {
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

  function clearSubmitted(request: SubmitConversationTaskRequest) {
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

  function restoreSubmitted(request: SubmitConversationTaskRequest) {
    const conversationId = request.conversationId ?? "";
    if (input.spaceId.value === request.spaceId && input.selectedConversationId.value === conversationId) {
      if (!input.prompt.value && !input.attachments.value.length) {
        input.prompt.value = request.prompt;
        input.attachments.value = [...(request.attachments ?? [])];
        drafts.save(request.spaceId, conversationId, {
          prompt: request.prompt,
          attachments: [...(request.attachments ?? [])],
        });
      }
      return;
    }
    if (!drafts.read(request.spaceId, conversationId)) {
      drafts.save(request.spaceId, conversationId, {
        prompt: request.prompt,
        attachments: [...(request.attachments ?? [])],
      });
    }
  }

  return { remember, restore, move, clearSubmitted, restoreSubmitted };
}
