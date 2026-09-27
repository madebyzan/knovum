import type { KnowledgeLocator } from "./schema/chunk";

export interface KnowledgePeekTarget {
  nodeId: string;
  nodeTitle: string;
  chunkId?: string;
  locator: KnowledgeLocator;
  text?: string;
}
