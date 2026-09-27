import { invokeCommand } from "../../../../foundation/ipc/invoke-command";

interface KnowledgeRetrievalRebuildOutput {
  rebuilt: boolean;
  indexed: number;
}

export function rebuildKnowledgeRetrievalIndex(spaceId: string): Promise<KnowledgeRetrievalRebuildOutput> {
  return invokeCommand("plugin:spaces|rebuild_knowledge_retrieval_index", { spaceId });
}
