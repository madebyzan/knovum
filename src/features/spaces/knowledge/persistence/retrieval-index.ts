import { invokeCommand } from "../../../../foundation/ipc/invoke-command";

export function indexKnowledgeRetrieval(
  spaceId: string,
  nodeIds?: readonly string[],
): Promise<{ indexed: number }> {
  return invokeCommand("plugin:spaces|index_knowledge", {
    spaceId,
    nodeIds: nodeIds === undefined ? null : [...nodeIds],
  });
}
