import { invokeCommand as invoke } from "../../../../foundation/ipc/invoke-command";
import {
  knowledgeSearchResponseSchema,
  type KnowledgeSearchResponse,
} from "../model/knowledge-search-response";

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
    void invoke<void>("plugin:spaces|cancel_knowledge_search", { jobId }).catch(console.error);
  };
  signal?.addEventListener("abort", cancel, { once: true });
  try {
    const source = await invoke<unknown>("plugin:spaces|search_knowledge_hybrid", {
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
