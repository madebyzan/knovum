import { basename } from "pathe";

import type { AssistantMessage, StartAssistantTaskInput } from "./conversation-ipc";

type VisibleAttachment = { title: string; isImage: boolean; previewSource: string };

export function pendingConversationMessage(
  request: StartAssistantTaskInput,
  identity: { id: string; conversationId: string; branchId: string },
  visibleAttachments: readonly VisibleAttachment[],
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
