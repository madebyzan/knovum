import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { isSpaceNodeOpenable, openSpaceNode } from "../canvas/selection-actions";
import type { ConversationLink } from "../contracts/generated/ConversationLink";
import { useSpaceStore } from "../session/space-store";

/**
 * Follows a link to a local file in a message. Rust resolves it — an Agent's deliverable to the
 * node it became — and opens a file it can vouch for in its system application; a Knovum document,
 * which has no file of its own, opens the way the canvas opens it.
 */
export async function followConversationLink(messageId: string, href: string): Promise<void> {
  const link = await invokeCommand<ConversationLink>("plugin:spaces|follow_conversation_link", {
    messageId,
    href,
  });
  if (link.kind === "opened") return;
  const store = useSpaceStore();
  const node = store.nodes.find(({ id }) => id === link.nodeId);
  if (isSpaceNodeOpenable(node)) await openSpaceNode(node);
  else store.requestReveal([link.nodeId], "focus");
}
