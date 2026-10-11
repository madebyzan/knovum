import { openSpaceNode } from "../canvas/selection-actions";
import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";
import { useSpaceStore } from "../session/space-store";
import { previewConversationUpload } from "./conversation-ipc";

export async function previewConversationAttachment(
  attachments: readonly ConversationAttachment[],
  key: string,
) {
  const attachment = attachments.find((item) =>
    item.kind === "file" ? key === `file:${item.path}` : key === `node:${item.nodeId}`,
  );
  if (!attachment) return;
  if (attachment.kind === "file") return previewConversationUpload(attachment.path);
  const store = useSpaceStore();
  const node = store.nodes.find(({ id }) => id === attachment.nodeId);
  if (!node) throw new Error("Attached Space resource is no longer available");
  return openSpaceNode(node);
}
