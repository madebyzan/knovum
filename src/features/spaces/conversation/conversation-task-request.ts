import { cloneDeep } from "es-toolkit";

import type { AgentExtensionSelection } from "../../../shared/contracts/generated/AgentExtensionSelection";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import type { Conversation } from "../contracts/generated/Conversation";
import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";
import type { SubmitConversationTaskRequest } from "../contracts/generated/SubmitConversationTaskRequest";

interface ConversationTaskRequestInput {
  spaceId: string | undefined;
  conversation: Conversation | undefined;
  prompt: string;
  target: TaskTarget | undefined;
  attachments: ConversationAttachment[];
  extensions: AgentExtensionSelection;
}

export function conversationTaskRequest(
  input: ConversationTaskRequestInput,
): SubmitConversationTaskRequest | undefined {
  const text = input.prompt.trim();
  if (!input.spaceId || !input.target || !text) return;
  return {
    spaceId: input.spaceId,
    ...(input.conversation ? { conversationId: input.conversation.id } : {}),
    ...(input.conversation?.branchId ? { branchId: input.conversation.branchId } : {}),
    prompt: text,
    target: input.target.type === "model" ? { ...input.target, temperature: 0.2 } : input.target,
    attachments: [...input.attachments],
    extensions: cloneDeep(input.extensions),
  };
}
