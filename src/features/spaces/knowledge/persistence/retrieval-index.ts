import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { KnowledgeIndexingProgress } from "../../contracts/generated/KnowledgeIndexingProgress";
import type { KnowledgeIndexState } from "../../contracts/generated/KnowledgeIndexState";

export interface KnowledgeIndexingInput {
  spaceId: string;
  nodeIds?: readonly string[] | undefined;
  onProgress: (progress: KnowledgeIndexingProgress) => void;
}

/** What semantic search can reach in this Space today, and what a pass would still have to do. */
export function readKnowledgeIndexState(spaceId: string): Promise<KnowledgeIndexState> {
  return invokeCommand("plugin:spaces|knowledge_index_state", { spaceId });
}

export function indexKnowledgeRetrieval({
  spaceId,
  nodeIds,
  onProgress,
}: KnowledgeIndexingInput): Promise<{ indexed: number }> {
  const channel = new Channel<KnowledgeIndexingProgress>();
  // Tauri Channel exposes one IPC receiver rather than a DOM EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = onProgress;
  return invokeCommand("plugin:spaces|index_knowledge", {
    spaceId,
    nodeIds: nodeIds === undefined ? null : [...nodeIds],
    onProgress: channel,
  });
}

/** A Space has one indexing pass, so cancelling it names the Space. */
export function cancelKnowledgeIndexing(spaceId: string): Promise<void> {
  return invokeCommand("plugin:spaces|cancel_knowledge_indexing", { spaceId });
}
