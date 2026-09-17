import { invoke } from "@tauri-apps/api/core";

import type { KnowledgeChunk } from "../model/knowledge-chunk";

export function listCurrentKnowledgeChunks(
  spaceId: string,
  nodeIds?: readonly string[],
): Promise<KnowledgeChunk[]> {
  return invoke("plugin:spaces|list_knowledge_chunks", {
    request: {
      spaceId,
      nodeIds: nodeIds === undefined ? null : [...nodeIds],
      currentOnly: true,
    },
  });
}
