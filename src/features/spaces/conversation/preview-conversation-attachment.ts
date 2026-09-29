import { openSpaceNode } from "../canvas/selection-actions";
import type { useSpaceStore } from "../session/space-store";
import { previewAssistantUpload, type AssistantAttachment } from "./conversation-ipc";

export async function previewConversationAttachment(
  attachments: readonly AssistantAttachment[],
  store: ReturnType<typeof useSpaceStore>,
  key: string,
) {
  const attachment = attachments.find((item) =>
    item.kind === "file" ? key === `file:${item.path}` : key === `node:${item.nodeId}`,
  );
  if (!attachment) return;
  if (attachment.kind === "file") return previewAssistantUpload(attachment.path);
  const node = store.nodes.find(({ id }) => id === attachment.nodeId);
  if (!node) throw new Error("Attached Space resource is no longer available");
  return openSpaceNode(store, node);
}
