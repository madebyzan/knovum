import type { KnowledgeSearchMatch } from "../knowledge/model/knowledge-search-match";

export type GlobalKnowledgeSearchMatch = KnowledgeSearchMatch & {
  kind: "content";
  spaceId: string;
  spaceTitle: string;
};

export interface GlobalStructureSearchMatch {
  kind: "structure";
  structureNodeId: string;
  nodeKind: "group" | "section";
  title: string;
  spaceId: string;
  spaceTitle: string;
}

export type GlobalSearchMatch = GlobalKnowledgeSearchMatch | GlobalStructureSearchMatch;
