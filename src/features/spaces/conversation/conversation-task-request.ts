import type { AssistantConversation, StartAssistantTaskInput } from "./conversation-ipc";
import type { ConversationTarget } from "./conversation-target";

export function conversationTaskRequest(input: {
  spaceId: string | undefined;
  conversation: AssistantConversation | undefined;
  prompt: string;
  target: ConversationTarget | undefined;
  attachments: NonNullable<StartAssistantTaskInput["attachments"]>;
}): StartAssistantTaskInput | undefined {
  const text = input.prompt.trim();
  if (!input.spaceId || !input.target || !text) return;
  return {
    spaceId: input.spaceId,
    ...(input.conversation ? { conversationId: input.conversation.id } : {}),
    ...(input.conversation?.branchId ? { branchId: input.conversation.branchId } : {}),
    prompt: text,
    target: input.target.type === "model" ? { ...input.target, temperature: 0.2 } : input.target,
    attachments: [...input.attachments],
  };
}
