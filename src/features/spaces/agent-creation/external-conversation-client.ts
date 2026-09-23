import { invoke } from "@tauri-apps/api/core";

import type { AgentKind } from "../../../foundation/ai/agent-models";
import type { ExternalConversationSummary } from "../contracts/generated/ExternalAgentConversation";
import type { ExternalConversationMessage } from "../contracts/generated/ExternalAgentMessage";

export type { ExternalConversationMessage, ExternalConversationSummary };

export function listExternalConversations(spaceId: string): Promise<ExternalConversationSummary[]> {
  return invoke("plugin:spaces|list_external_agent_conversations", { spaceId });
}

export function listExternalMessages(conversationId: string): Promise<ExternalConversationMessage[]> {
  return invoke("plugin:spaces|list_external_agent_messages", { conversationId });
}

export function startExternalConversation(input: {
  spaceId: string;
  runtimeKind: AgentKind;
  title: string;
  maxDurationSeconds: number;
}): Promise<ExternalConversationSummary> {
  return invoke("plugin:spaces|start_external_agent_conversation", { request: input });
}

export function sendExternalMessage(
  conversationId: string,
  prompt: string,
  attachments: readonly string[],
): Promise<void> {
  return invoke("plugin:spaces|send_external_agent_message", { conversationId, prompt, attachments });
}

export function setExternalSessionOption(
  conversationId: string,
  optionId: string,
  value: string | boolean,
): Promise<ExternalConversationSummary> {
  return invoke("plugin:spaces|set_external_agent_session_option", {
    conversationId,
    optionId,
    value,
  });
}

export function setExternalSessionMode(
  conversationId: string,
  modeId: string,
): Promise<ExternalConversationSummary> {
  return invoke("plugin:spaces|set_external_agent_session_mode", { conversationId, modeId });
}

export function closeExternalConversation(conversationId: string): Promise<void> {
  return invoke("plugin:spaces|close_external_agent_conversation", { conversationId });
}

export function deleteExternalConversation(conversationId: string): Promise<void> {
  return invoke("plugin:spaces|delete_external_agent_conversation", { conversationId });
}
