import type { ActiveAssistantRun } from "../contracts/generated/ActiveAssistantRun";
export type { ActiveAssistantRun } from "../contracts/generated/ActiveAssistantRun";
import type { SubmitAssistantTaskRequest as StartAssistantTaskInput } from "../contracts/generated/SubmitAssistantTaskRequest";
export type { SubmitAssistantTaskRequest as StartAssistantTaskInput } from "../contracts/generated/SubmitAssistantTaskRequest";

export type { AssistantAttachment } from "../contracts/generated/AssistantAttachment";
import type { InspectedAssistantUpload } from "../contracts/generated/InspectedAssistantUpload";
export type { InspectedAssistantUpload } from "../contracts/generated/InspectedAssistantUpload";
import { Channel } from "@tauri-apps/api/core";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { AssistantConversation } from "../contracts/generated/AssistantConversation";
import type { AssistantMessage } from "../contracts/generated/AssistantMessage";
import type { AssistantTaskAccepted } from "../contracts/generated/AssistantTaskAccepted";
import type { AssistantTaskUpdate } from "../contracts/generated/AssistantTaskUpdate";
import type { PendingApproval } from "../contracts/generated/PendingApproval";

export type { AssistantConversation } from "../contracts/generated/AssistantConversation";
export type { AssistantMessage, AssistantTaskAccepted };

export function listActiveAssistantRuns(spaceId?: string): Promise<ActiveAssistantRun[]> {
  return invoke("plugin:spaces|list_active_assistant_runs", { spaceId });
}

export function getPendingAssistantPermission(
  runId: string,
): Promise<Extract<AssistantTaskUpdate, { type: "permission_requested" }> | null> {
  return invoke("plugin:spaces|get_pending_assistant_permission", { runId });
}

export function listAssistantApprovals(spaceId: string, runId: string): Promise<PendingApproval[]> {
  return invoke<PendingApproval[]>("plugin:spaces|list_pending_action_approvals", { spaceId }).then((items) =>
    items.filter((item) => item.runId === runId),
  );
}

export function decideAssistantApproval(approvalId: string, approved: boolean): Promise<void> {
  return invoke("plugin:spaces|decide_action_approval", {
    approvalId,
    decision: approved ? "approve" : "reject",
  });
}

export function getAssistantTaskError(runId: string): Promise<string | null> {
  return invoke("plugin:spaces|get_assistant_task_error", { runId });
}

export function inspectAssistantUpload(path: string): Promise<InspectedAssistantUpload> {
  return invoke("plugin:spaces|inspect_assistant_upload", { path });
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
  onUpdate: (update: AssistantTaskUpdate) => void,
): Promise<{ accepted: AssistantTaskAccepted; settled: Promise<string> }> {
  let finish!: (state: string) => void;
  const settled = new Promise<string>((resolve) => (finish = resolve));
  const updates = new Channel<AssistantTaskUpdate>((update) => {
    if (update.type === "finished") finish(update.state);
    else onUpdate(update);
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

export function archiveAssistantConversation(conversationId: string): Promise<void> {
  return invoke("plugin:spaces|archive_assistant_conversation", { conversationId });
}

export function unarchiveAssistantConversation(conversationId: string): Promise<void> {
  return invoke("plugin:spaces|unarchive_assistant_conversation", { conversationId });
}

export function deleteAssistantConversation(conversationId: string): Promise<void> {
  return invoke("plugin:spaces|delete_assistant_conversation", { conversationId });
}

export function loadAssistantBranchMessages(conversationId: string, branchId: string): Promise<unknown> {
  return invoke("plugin:spaces|load_assistant_branch_messages", { conversationId, branchId });
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
