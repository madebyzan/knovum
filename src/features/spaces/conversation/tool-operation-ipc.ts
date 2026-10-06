import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { ConversationToolOperation } from "../contracts/generated/ConversationToolOperation";
import type { McpUserInteraction } from "../contracts/generated/McpUserInteraction";
import type { ToolOperationDecision } from "../contracts/generated/ToolOperationDecision";
export const listConversationToolOperations = (runId: string) =>
  invokeCommand<ConversationToolOperation[]>("plugin:spaces|list_conversation_tool_operations", { runId });
export const decideConversationToolOperation = (operationId: string, decision: ToolOperationDecision) =>
  invokeCommand<void>("plugin:spaces|decide_conversation_tool_operation", { operationId, decision });
export const listMcpUserInteractions = (runId: string) =>
  invokeCommand<McpUserInteraction[]>("plugin:spaces|list_mcp_user_interactions", { runId });
export const answerMcpUserInteraction = (
  id: string,
  response: {
    action: "accept" | "decline" | "cancel";
    content?: Record<string, string | number | boolean | string[]>;
  },
) => invokeCommand<void>("plugin:spaces|answer_mcp_user_interaction", { id, response });
