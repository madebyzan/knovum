import { basename } from "pathe";

import type { AssistantMessage } from "../contracts/generated/AssistantMessage";
import type { SubmitAssistantTaskRequest } from "../contracts/generated/SubmitAssistantTaskRequest";

export interface VisibleConversationAttachment {
  title: string;
  isImage: boolean;
  previewSource: string;
}

export function pendingConversationMessage(
  request: SubmitAssistantTaskRequest,
  identity: Pick<AssistantMessage, "id" | "conversationId" | "branchId">,
  visibleAttachments: readonly VisibleConversationAttachment[],
): AssistantMessage {
  const attachments = (request.attachments ?? []).map((attachment, index) => {
    const visible = visibleAttachments[index];
    const thumbnail = visible?.previewSource || null;
    const kind = visible?.isImage ? "image" : "file";
    return attachment.kind === "file"
      ? { source: "file" as const, title: visible?.title ?? basename(attachment.path), kind, thumbnail }
      : {
          source: "node" as const,
          nodeId: attachment.nodeId,
          title: visible?.title ?? attachment.nodeId,
          kind,
          thumbnail,
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
