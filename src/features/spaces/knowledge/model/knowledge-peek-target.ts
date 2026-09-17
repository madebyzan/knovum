import type { KnowledgeLocator } from "./schema";

export interface KnowledgePeekTarget {
  nodeId: string;
  nodeTitle: string;
  chunkId?: string;
  locator: KnowledgeLocator;
  text?: string;
}
