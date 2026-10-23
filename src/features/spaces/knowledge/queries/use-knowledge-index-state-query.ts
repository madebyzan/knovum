import type { MaybeRefOrGetter } from "vue";

import { readKnowledgeIndexState } from "../persistence/retrieval-index";
import { SPACE_KNOWLEDGE_QUERY_KEYS } from "./query-options";
import { useSpaceQuery } from "./use-space-query";

export function useKnowledgeIndexStateQuery(spaceId: MaybeRefOrGetter<string | null | undefined>) {
  return useSpaceQuery(spaceId, SPACE_KNOWLEDGE_QUERY_KEYS.indexState, readKnowledgeIndexState);
}
