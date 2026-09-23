import { Channel, invoke } from "@tauri-apps/api/core";

import type { ExecutionTarget } from "../../../foundation/ai/ai-models";
import type { ConversationSummary } from "../contracts/generated/AssistantConversation";
import type { AssistantMessage } from "../contracts/generated/AssistantMessage";
import type { AssistantMode } from "../contracts/generated/AssistantMode";
import type { AssistantTaskAccepted } from "../contracts/generated/AssistantTaskAccepted";
import type { AssistantTaskUpdate } from "../contracts/generated/AssistantTaskUpdate";

export type AssistantConversation = ConversationSummary;
export type { AssistantMessage, AssistantMode, AssistantTaskAccepted };

export interface StartAssistantTaskInput {
  spaceId: string;
  conversationId?: string;
  branchId?: string;
  prompt: string;
  mode: AssistantMode;
  target: ExecutionTarget;
  maxTurns: number;
  maxToolCalls: number;
  maxTotalTokens: number;
  maxDurationSeconds: number;
  attachments?: Array<{ nodeId: string; sourceRevision: string; role: string; locator?: unknown }>;
}

export function listAssistantConversations(
  spaceId: string,
  query = "",
  archived = false,
): Promise<AssistantConversation[]> {
  return invoke("plugin:spaces|list_assistant_conversations", { spaceId, query, archived });
}

export function listAssistantMessages(
  conversationId: string,
  branchId: string,
  before?: Pick<AssistantMessage, "createdAt" | "id">,
): Promise<AssistantMessage[]> {
  return invoke("plugin:spaces|list_assistant_messages", {
    conversationId,
    branchId,
    beforeCreatedAt: before?.createdAt,
    beforeId: before?.id,
    limit: 100,
  });
}

export async function startAssistantTask(
  request: StartAssistantTaskInput,
  onContent: (text: string) => void,
): Promise<{ accepted: AssistantTaskAccepted; settled: Promise<string> }> {
  let finish!: (state: string) => void;
  const settled = new Promise<string>((resolve) => (finish = resolve));
  const updates = new Channel<AssistantTaskUpdate>((update) => {
    if (update.type === "content_delta") onContent(update.text);
    else finish(update.state);
  });
  const accepted = await invoke<AssistantTaskAccepted>("plugin:spaces|start_assistant_task", {
    request,
    updates,
  });
  return { accepted, settled };
}

export function cancelAssistantTask(runId: string): Promise<void> {
  return invoke("plugin:spaces|cancel_run", { runId });
}

export function renameAssistantConversation(conversationId: string, title: string): Promise<void> {
  return invoke("plugin:spaces|rename_assistant_conversation", { conversationId, title });
}

export function archiveAssistantConversation(conversationId: string, archived = true): Promise<void> {
  return invoke("plugin:spaces|archive_assistant_conversation", { conversationId, archived });
}

export function deleteAssistantConversation(conversationId: string): Promise<void> {
  return invoke("plugin:spaces|delete_assistant_conversation", { conversationId });
}

export function exportAssistantConversation(conversationId: string, branchId: string): Promise<unknown> {
  return invoke("plugin:spaces|export_assistant_conversation", { conversationId, branchId });
}

export function branchAssistantConversation(
  conversationId: string,
  sourceBranchId: string,
  throughMessageId: string,
  includeBoundary = true,
): Promise<string> {
  return invoke("plugin:spaces|branch_assistant_conversation", {
    conversationId,
    sourceBranchId,
    throughMessageId,
    includeBoundary,
  });
}
