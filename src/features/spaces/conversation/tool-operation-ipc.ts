import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { ConversationToolOperation } from "../contracts/generated/ConversationToolOperation";
import type { McpUserInteraction } from "../contracts/generated/McpUserInteraction";
import type { ToolOperationDecision } from "../contracts/generated/ToolOperationDecision";
export interface McpInteractionResponse {
  action: "accept" | "decline" | "cancel";
  content?: Record<string, string | number | boolean | string[]>;
}

export function listConversationToolOperations(runId: string) {
  return invokeCommand<ConversationToolOperation[]>("plugin:spaces|list_conversation_tool_operations", {
    runId,
  });
}
export function decideConversationToolOperation(operationId: string, decision: ToolOperationDecision) {
  return invokeCommand<void>("plugin:spaces|decide_conversation_tool_operation", { operationId, decision });
}
export function listMcpUserInteractions(runId: string) {
  return invokeCommand<McpUserInteraction[]>("plugin:spaces|list_mcp_user_interactions", { runId });
}
export function answerMcpUserInteraction(interactionId: string, response: McpInteractionResponse) {
  return invokeCommand<void>("plugin:spaces|answer_mcp_user_interaction", { interactionId, response });
}
