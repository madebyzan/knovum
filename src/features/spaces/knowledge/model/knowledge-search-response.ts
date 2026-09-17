import type { KnowledgeSearchMatch } from "./knowledge-search-match";

export interface KnowledgeSearchResponse {
  matches: KnowledgeSearchMatch[];
  semantic: "ready" | "unavailable" | "empty";
}
