import { invoke } from "@tauri-apps/api/core";

export function indexKnowledgeRetrieval(
  spaceId: string,
  nodeIds?: readonly string[],
): Promise<{ indexed: number }> {
  return invoke("plugin:spaces|index_knowledge", {
    spaceId,
    nodeIds: nodeIds === undefined ? null : [...nodeIds],
  });
}
