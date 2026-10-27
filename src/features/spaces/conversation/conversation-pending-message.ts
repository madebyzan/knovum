import { basename } from "pathe";

import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { SubmitConversationTaskRequest } from "../contracts/generated/SubmitConversationTaskRequest";
import type { ConversationAttachmentPresentation } from "./conversation-attachment-presentation";

export type VisibleConversationAttachment = Pick<
  ConversationAttachmentPresentation,
  "title" | "isImage" | "previewSource"
>;

export function pendingConversationMessage(
  request: SubmitConversationTaskRequest,
  identity: Pick<ConversationMessage, "id" | "conversationId" | "branchId">,
  visibleAttachments: readonly VisibleConversationAttachment[],
): ConversationMessage {
  const attachments = (request.attachments ?? []).map((attachment, index) => {
    const visible = visibleAttachments[index];
    const kind = visible?.isImage ? "image" : "file";
    return attachment.kind === "file"
      ? {
          source: "file" as const,
          title: visible?.title ?? basename(attachment.path),
          kind,
          thumbnail: visible?.previewSource || null,
        }
      : {
          source: "node" as const,
          nodeId: attachment.nodeId,
          title: visible?.title ?? attachment.nodeId,
          kind,
        };
  });
  return {
    id: identity.id,
    conversationId: identity.conversationId,
    branchId: identity.branchId,
    role: "user",
    content: { parts: [{ type: "text", text: request.prompt }], attachments, citations: [] },
    createdAt: Date.now(),
  };
}
