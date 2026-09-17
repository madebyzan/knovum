import { invoke } from "@tauri-apps/api/core";

import type { KnowledgeSearchResponse } from "../model/knowledge-search-response";

export function searchSpaceKnowledgeHybrid(
  spaceId: string,
  query: string,
  nodeIds?: readonly string[],
): Promise<KnowledgeSearchResponse> {
  return invoke("plugin:spaces|search_knowledge_hybrid", {
    request: {
      spaceId,
      query,
      nodeIds: nodeIds === undefined ? null : [...nodeIds],
    },
  });
}
