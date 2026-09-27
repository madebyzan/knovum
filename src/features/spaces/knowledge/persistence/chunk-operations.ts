import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { KnowledgeChunk } from "../model/knowledge-chunk";

export function listCurrentKnowledgeChunks(
  spaceId: string,
  nodeIds?: readonly string[],
): Promise<KnowledgeChunk[]> {
  return invokeCommand("plugin:spaces|list_knowledge_chunks", {
    request: {
      spaceId,
      nodeIds: nodeIds === undefined ? null : [...nodeIds],
      currentOnly: true,
    },
  });
}
