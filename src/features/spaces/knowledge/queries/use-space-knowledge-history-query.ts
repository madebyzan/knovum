import { toValue, type MaybeRefOrGetter } from "vue";

import { listKnowledgeEvents } from "../persistence/knowledge-operations";
import { SPACE_KNOWLEDGE_QUERY_KEYS } from "./query-options";
import { useSpaceQuery } from "./use-space-query";

export function useSpaceKnowledgeHistoryQuery(
  spaceId: MaybeRefOrGetter<string | null | undefined>,
  nodeId: MaybeRefOrGetter<string>,
) {
  return useSpaceQuery(
    spaceId,
    (id) => SPACE_KNOWLEDGE_QUERY_KEYS.history(id, toValue(nodeId)),
    (id) =>
      listKnowledgeEvents({
        spaceId: id,
        entityKind: "node",
        entityId: toValue(nodeId),
        limit: 30,
      }),
    () => Boolean(toValue(nodeId)),
  );
}
