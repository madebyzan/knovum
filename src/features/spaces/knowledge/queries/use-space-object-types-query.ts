import type { MaybeRefOrGetter } from "vue";

import { listObjectTypes } from "../persistence/knowledge-operations";
import { SPACE_KNOWLEDGE_QUERY_KEYS } from "./query-options";
import { useSpaceQuery } from "./use-space-query";

export function useSpaceObjectTypesQuery(spaceId: MaybeRefOrGetter<string | null | undefined>) {
  return useSpaceQuery(spaceId, SPACE_KNOWLEDGE_QUERY_KEYS.objectTypes, listObjectTypes);
}
