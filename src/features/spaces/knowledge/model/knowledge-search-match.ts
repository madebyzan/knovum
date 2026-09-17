import type { KnowledgeChunkKind, KnowledgeLocator } from "./schema";

export interface KnowledgeSearchMatch {
  chunkId: string;
  nodeId: string;
  nodeKind: string;
  nodeTitle: string;
  chunkKind: KnowledgeChunkKind;
  metadata?: Record<string, string | number | boolean | null>;
  sourceRevision: string;
  locator: KnowledgeLocator;
  snippet: string;
  text: string;
  score: number;
  lexicalScore?: number;
  semanticScore?: number;
  spaceId?: string;
  spaceTitle?: string;
}
