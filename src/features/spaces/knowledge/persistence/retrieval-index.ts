import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { KnowledgeIndexingProgress } from "../../contracts/generated/KnowledgeIndexingProgress";

export interface KnowledgeIndexingInput {
  spaceId: string;
  jobId: string;
  nodeIds?: readonly string[] | undefined;
  onProgress: (progress: KnowledgeIndexingProgress) => void;
}

export function indexKnowledgeRetrieval({
  spaceId,
  jobId,
  nodeIds,
  onProgress,
}: KnowledgeIndexingInput): Promise<{ indexed: number }> {
  const channel = new Channel<KnowledgeIndexingProgress>();
  // Tauri Channel exposes one IPC receiver rather than a DOM EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = onProgress;
  return invokeCommand("plugin:spaces|index_knowledge", {
    spaceId,
    jobId,
    nodeIds: nodeIds === undefined ? null : [...nodeIds],
    onProgress: channel,
  });
}

export function cancelKnowledgeIndexing(jobId: string): Promise<void> {
  return invokeCommand("plugin:spaces|cancel_knowledge_indexing", { jobId });
}
