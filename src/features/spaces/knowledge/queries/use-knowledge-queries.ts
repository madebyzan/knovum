import { useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";

import {
  listKnowledgeEvents,
  listKnowledgeHealthIssues,
  listObjectTypes,
  listProvenance,
} from "../persistence/knowledge-operations";
import { spaceKnowledgeQueryKeys } from "./query-options";

function spaceIdValue(spaceId: MaybeRefOrGetter<string | null | undefined>) {
  return toValue(spaceId) ?? "none";
}

function useSpaceQuery<T>(
  spaceId: MaybeRefOrGetter<string | null | undefined>,
  keyFactory: (spaceId: string) => readonly unknown[],
  fetcher: (spaceId: string) => Promise<T>,
  extraEnabled?: () => boolean,
) {
  return useQuery<T>(
    computed(() => {
      const id = spaceIdValue(spaceId);
      return {
        queryKey: keyFactory(id),
        queryFn: () => fetcher(id),
        enabled: Boolean(toValue(spaceId)) && (extraEnabled ? extraEnabled() : true),
      };
    }),
  );
}

export function useSpaceObjectTypesQuery(spaceId: MaybeRefOrGetter<string | null | undefined>) {
  return useSpaceQuery(spaceId, spaceKnowledgeQueryKeys.objectTypes, listObjectTypes);
}

export function useSpaceKnowledgeHealthQuery(spaceId: MaybeRefOrGetter<string | null | undefined>) {
  return useSpaceQuery(spaceId, spaceKnowledgeQueryKeys.health, listKnowledgeHealthIssues);
}

export function useSpaceKnowledgeHistoryQuery(
  spaceId: MaybeRefOrGetter<string | null | undefined>,
  nodeId: MaybeRefOrGetter<string>,
) {
  return useSpaceQuery(
    spaceId,
    (id) => spaceKnowledgeQueryKeys.history(id, toValue(nodeId)),
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

export function useSpaceProvenanceQuery(
  spaceId: MaybeRefOrGetter<string | null | undefined>,
  targetKind: "node" | "relation",
  targetId: MaybeRefOrGetter<string>,
) {
  return useSpaceQuery(
    spaceId,
    (id) => spaceKnowledgeQueryKeys.provenance(id, targetKind, toValue(targetId)),
    (id) => listProvenance(id, targetKind, toValue(targetId)),
    () => Boolean(toValue(targetId)),
  );
}
