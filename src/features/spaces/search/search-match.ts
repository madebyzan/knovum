import type { KnowledgeSearchMatch } from "../knowledge/model/schema/search-match";

type GlobalKnowledgeSearchMatch = KnowledgeSearchMatch & {
  kind: "content";
  spaceId: string;
  spaceTitle: string;
};

interface GlobalStructureSearchMatch {
  kind: "structure";
  structureNodeId: string;
  nodeKind: "group" | "section";
  title: string;
  spaceId: string;
  spaceTitle: string;
}

export type GlobalSearchMatch = GlobalKnowledgeSearchMatch | GlobalStructureSearchMatch;
