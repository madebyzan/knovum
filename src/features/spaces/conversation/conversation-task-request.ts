import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import type { AssistantAttachment } from "../contracts/generated/AssistantAttachment";
import type { AssistantConversation } from "../contracts/generated/AssistantConversation";
import type { AssistantMode } from "../contracts/generated/AssistantMode";
import type { SubmitAssistantTaskRequest } from "../contracts/generated/SubmitAssistantTaskRequest";

interface ConversationTaskRequestInput {
  mode: AssistantMode;
  spaceId: string | undefined;
  conversation: AssistantConversation | undefined;
  prompt: string;
  target: TaskTargetRef | undefined;
  attachments: AssistantAttachment[];
}

export function conversationTaskRequest(
  input: ConversationTaskRequestInput,
): SubmitAssistantTaskRequest | undefined {
  const text = input.prompt.trim();
  if (!input.spaceId || !input.target || !text) return;
  return {
    mode: input.target.type === "model" ? input.mode : "execute",
    spaceId: input.spaceId,
    ...(input.conversation ? { conversationId: input.conversation.id } : {}),
    ...(input.conversation?.branchId ? { branchId: input.conversation.branchId } : {}),
    prompt: text,
    target: input.target.type === "model" ? { ...input.target, temperature: 0.2 } : input.target,
    attachments: [...input.attachments],
  };
}
