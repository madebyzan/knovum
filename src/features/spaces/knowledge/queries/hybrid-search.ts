import { invoke } from "@tauri-apps/api/core";

import {
  knowledgeSearchResponseSchema,
  type KnowledgeSearchResponse,
} from "../model/knowledge-search-response";

export async function searchSpaceKnowledgeHybrid(
  spaceId: string,
  query: string,
  nodeIds?: readonly string[],
): Promise<KnowledgeSearchResponse> {
  const source = await invoke<unknown>("plugin:spaces|search_knowledge_hybrid", {
    request: {
      spaceId,
      query,
      nodeIds: nodeIds === undefined ? null : [...nodeIds],
    },
  });
  return knowledgeSearchResponseSchema.parse(source);
}
