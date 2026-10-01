import { error as logError } from "@tauri-apps/plugin-log";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import { knowledgeSearchResponseSchema, type KnowledgeSearchResponse } from "../model/schema/search-response";

interface HybridSearchOptions {
  nodeIds?: readonly string[];
  semantic?: boolean;
  signal?: AbortSignal;
}

export async function searchSpaceKnowledgeHybrid(
  spaceId: string,
  query: string,
  { nodeIds, semantic = false, signal }: HybridSearchOptions = {},
): Promise<KnowledgeSearchResponse> {
  signal?.throwIfAborted();
  const jobId = crypto.randomUUID();
  const cancel = () => {
    void invokeCommand<void>("plugin:spaces|cancel_knowledge_search", { jobId }).catch((error) => {
      void logError(`Knowledge search cancellation failed: ${String(error)}`);
    });
  };
  signal?.addEventListener("abort", cancel, { once: true });
  try {
    const source = await invokeCommand<unknown>("plugin:spaces|search_knowledge_hybrid", {
      jobId,
      request: {
        spaceId,
        query,
        semantic,
        nodeIds: nodeIds === undefined ? null : [...nodeIds],
      },
    });
    signal?.throwIfAborted();
    return knowledgeSearchResponseSchema.parse(source);
  } finally {
    signal?.removeEventListener("abort", cancel);
  }
}
