import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { ActiveConversationRun } from "../contracts/generated/ActiveConversationRun";
import type { Conversation } from "../contracts/generated/Conversation";
import type { ConversationDraft } from "../contracts/generated/ConversationDraft";
import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { ConversationTaskAccepted } from "../contracts/generated/ConversationTaskAccepted";
import type { ConversationTaskUpdate } from "../contracts/generated/ConversationTaskUpdate";
import type { InspectedConversationUpload } from "../contracts/generated/InspectedConversationUpload";
import type { RunEvent } from "../contracts/generated/RunEvent";
import type { SaveConversationDraftRequest } from "../contracts/generated/SaveConversationDraftRequest";
import type { SubmitConversationTaskRequest } from "../contracts/generated/SubmitConversationTaskRequest";
import { requestRunControl } from "../runs/run-control";

type ConversationTaskFinishedUpdate = Extract<ConversationTaskUpdate, { type: "finished" }>;

export function loadConversationDraft(
  spaceId: string,
  conversationId: string,
): Promise<ConversationDraft | null> {
  return invokeCommand("plugin:spaces|load_conversation_draft", { spaceId, conversationId });
}

export function saveConversationDraft(request: SaveConversationDraftRequest): Promise<ConversationDraft> {
  return invokeCommand("plugin:spaces|save_conversation_draft", { request });
}

export function removeConversationDraft(spaceId: string, conversationId: string): Promise<void> {
  return invokeCommand("plugin:spaces|remove_conversation_draft", { spaceId, conversationId });
}

export function moveConversationDraft(
  spaceId: string,
  fromConversationId: string,
  toConversationId: string,
): Promise<void> {
  return invokeCommand("plugin:spaces|move_conversation_draft", {
    spaceId,
    fromConversationId,
    toConversationId,
  });
}

export function listConversationRunEvents(runId: string, afterSequence = 0): Promise<RunEvent[]> {
  return invokeCommand("plugin:spaces|list_run_events", { runId, afterSequence });
}

export function listActiveConversationRuns(spaceId?: string): Promise<ActiveConversationRun[]> {
  return invokeCommand("plugin:spaces|list_active_conversation_runs", { spaceId });
}

export function getConversationTaskError(runId: string): Promise<string | null> {
  return invokeCommand("plugin:spaces|get_conversation_task_error", { runId });
}

export function inspectConversationUpload(path: string): Promise<InspectedConversationUpload> {
  return invokeCommand("plugin:spaces|inspect_conversation_upload", { path });
}

export function previewConversationUpload(path: string): Promise<void> {
  return invokeCommand("plugin:spaces|preview_conversation_upload", { path });
}

export function listConversations(spaceId: string, query = "", archived = false): Promise<Conversation[]> {
  return invokeCommand("plugin:spaces|list_conversations", { spaceId, query, archived });
}

export function listConversationMessages(
  conversationId: string,
  branchId: string,
  before?: Pick<ConversationMessage, "createdAt" | "id">,
): Promise<ConversationMessage[]> {
  return invokeCommand("plugin:spaces|list_conversation_messages", {
    conversationId,
    branchId,
    beforeCreatedAt: before?.createdAt,
    beforeId: before?.id,
    limit: 100,
  });
}

export function listConversationBranches(conversationId: string): Promise<string[]> {
  return invokeCommand("plugin:spaces|list_conversation_branches", { conversationId });
}

export function selectConversationBranch(conversationId: string, branchId: string): Promise<void> {
  return invokeCommand("plugin:spaces|select_conversation_branch", { conversationId, branchId });
}

export async function startConversationTask(
  request: SubmitConversationTaskRequest,
  onUpdate: (update: ConversationTaskUpdate) => void,
): Promise<{
  accepted: ConversationTaskAccepted;
  settled: Promise<ConversationTaskFinishedUpdate>;
}> {
  let finish!: (update: ConversationTaskFinishedUpdate) => void;
  const settled = new Promise<ConversationTaskFinishedUpdate>((resolve) => (finish = resolve));
  const updates = new Channel<ConversationTaskUpdate>((update) => {
    if (update.type === "finished") finish(update);
    else onUpdate(update);
  });
  const accepted = await invokeCommand<ConversationTaskAccepted>("plugin:spaces|start_conversation_task", {
    request,
    updates,
  });
  return { accepted, settled };
}

export function cancelConversationTask(runId: string): Promise<void> {
  return requestRunControl("cancel", runId);
}

export function pauseConversationTask(runId: string): Promise<void> {
  return requestRunControl("pause", runId);
}

export function renameConversation(conversationId: string, title: string): Promise<void> {
  return invokeCommand("plugin:spaces|rename_conversation", { conversationId, title });
}

export function archiveConversation(conversationId: string): Promise<void> {
  return invokeCommand("plugin:spaces|archive_conversation", { conversationId });
}

export function unarchiveConversation(conversationId: string): Promise<void> {
  return invokeCommand("plugin:spaces|unarchive_conversation", { conversationId });
}

export function deleteConversation(conversationId: string): Promise<void> {
  return invokeCommand("plugin:spaces|delete_conversation", { conversationId });
}

export function loadConversationBranchMessages(conversationId: string, branchId: string): Promise<unknown> {
  return invokeCommand("plugin:spaces|load_conversation_branch_messages", { conversationId, branchId });
}

interface BranchConversationInput {
  conversationId: string;
  sourceBranchId: string;
  throughMessageId: string;
  includeBoundary?: boolean;
}

export function branchConversation({
  conversationId,
  sourceBranchId,
  throughMessageId,
  includeBoundary = true,
}: BranchConversationInput): Promise<string> {
  return invokeCommand("plugin:spaces|branch_conversation", {
    conversationId,
    sourceBranchId,
    throughMessageId,
    includeBoundary,
  });
}
