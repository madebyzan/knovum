import { invoke } from "@tauri-apps/api/core";

import type { GroundedMessage } from "../model/grounded-message";
import type { GroundedThread } from "../model/grounded-thread";

export function listGroundedThreads(spaceId: string): Promise<GroundedThread[]> {
  return invoke("plugin:spaces|list_grounded_threads", { spaceId });
}

export function updateGroundedThreadTitle(threadId: string, title: string): Promise<void> {
  return invoke("plugin:spaces|update_grounded_thread_title", { threadId, title });
}

export function deleteGroundedThread(threadId: string): Promise<void> {
  return invoke("plugin:spaces|delete_grounded_thread", { threadId });
}

export function listGroundedMessages(threadId: string): Promise<GroundedMessage[]> {
  return invoke("plugin:spaces|list_grounded_messages", { threadId });
}
