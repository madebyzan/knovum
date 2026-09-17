import { invoke } from "@tauri-apps/api/core";

interface KnowledgeRetrievalRebuildOutput {
  rebuilt: boolean;
  indexed: number;
}

export function rebuildKnowledgeRetrievalIndex(spaceId: string): Promise<KnowledgeRetrievalRebuildOutput> {
  return invoke("plugin:spaces|rebuild_knowledge_retrieval_index", { spaceId });
}
