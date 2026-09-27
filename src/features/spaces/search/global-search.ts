import { z } from "zod";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { knowledgeSearchMatchSchema } from "../knowledge/model/schema/search-match";
import type { GlobalSearchMatch } from "./search-match";
import { markSpaceSubstring, searchTerms } from "./search-snippet";

const globalKnowledgeSearchMatchSchema = knowledgeSearchMatchSchema.extend({
  spaceId: z.string().min(1),
  spaceTitle: z.string(),
});

const globalStructureSearchMatchSchema = z
  .object({
    structureNodeId: z.string().min(1),
    nodeKind: z.enum(["group", "section"]),
    title: z.string(),
    spaceId: z.string().min(1),
    spaceTitle: z.string(),
  })
  .strict();

export async function searchAcrossSpaces(value: string): Promise<GlobalSearchMatch[]> {
  if (!value.trim()) return [];
  const request = { request: { query: value } };
  const [knowledgeSource, structureSource] = await Promise.all([
    invokeCommand<unknown>("plugin:spaces|search_knowledge_global", request),
    invokeCommand<unknown>("plugin:spaces|search_structure_global", request),
  ]);
  const knowledge = z.array(globalKnowledgeSearchMatchSchema).parse(knowledgeSource);
  const structure = z.array(globalStructureSearchMatchSchema).parse(structureSource);
  const terms = searchTerms(value);
  return [
    ...structure.map((row) => Object.assign({}, row, { kind: "structure" as const })),
    ...knowledge.map((row) =>
      Object.assign({}, row, {
        kind: "content" as const,
        snippet: row.snippet === row.text ? markSpaceSubstring(row.text, terms) : row.snippet,
      }),
    ),
  ];
}
