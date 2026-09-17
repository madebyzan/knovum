import { invoke } from "@tauri-apps/api/core";

import type { KnowledgeSearchMatch } from "../knowledge/model/knowledge-search-match";
import { markSpaceSubstring, searchTerms } from "./search-snippet";

export interface GlobalKnowledgeSearchMatch extends KnowledgeSearchMatch {
  spaceId: string;
  spaceTitle: string;
}

export async function searchKnowledgeAcrossSpaces(value: string): Promise<GlobalKnowledgeSearchMatch[]> {
  if (!value.trim()) return [];
  const rows: GlobalKnowledgeSearchMatch[] = await invoke<GlobalKnowledgeSearchMatch[]>(
    "plugin:spaces|search_knowledge_global",
    {
      request: { query: value },
    },
  );
  const terms = searchTerms(value);
  return rows.map((row) =>
    row.snippet === row.text ? Object.assign({}, row, { snippet: markSpaceSubstring(row.text, terms) }) : row,
  );
}
