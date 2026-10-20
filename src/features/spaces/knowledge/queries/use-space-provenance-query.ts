import { toValue, type MaybeRefOrGetter } from "vue";

import { listProvenance } from "../persistence/knowledge-operations";
import { SPACE_KNOWLEDGE_QUERY_KEYS } from "./query-options";
import { useSpaceQuery } from "./use-space-query";

export function useSpaceProvenanceQuery(
  spaceId: MaybeRefOrGetter<string | null | undefined>,
  targetKind: "node" | "relation",
  targetId: MaybeRefOrGetter<string>,
) {
  return useSpaceQuery(
    spaceId,
    (id) => SPACE_KNOWLEDGE_QUERY_KEYS.provenance(id, targetKind, toValue(targetId)),
    (id) => listProvenance(id, targetKind, toValue(targetId)),
    () => Boolean(toValue(targetId)),
  );
}
