import { useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";

import { listRecentlyDeletedNodes } from "../../nodes/node-repository";
import {
  listKnowledgeEvents,
  listKnowledgeHealthIssues,
  listObjectTypes,
  listProvenance,
} from "../persistence/knowledge-repository";
import { spaceKnowledgeQueryKeys } from "./query-options";

function spaceIdValue(spaceId: MaybeRefOrGetter<string | null | undefined>) {
  return toValue(spaceId) ?? "none";
}

function spaceQuery<T>(
  spaceId: MaybeRefOrGetter<string | null | undefined>,
  keyFactory: (id: string) => readonly unknown[],
  fetcher: (id: string) => Promise<T>,
  extraEnabled?: () => boolean,
) {
  return useQuery(
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
  return spaceQuery(spaceId, spaceKnowledgeQueryKeys.objectTypes, listObjectTypes);
}

export function useSpaceKnowledgeHealthQuery(spaceId: MaybeRefOrGetter<string | null | undefined>) {
  return spaceQuery(spaceId, spaceKnowledgeQueryKeys.health, listKnowledgeHealthIssues);
}

export function useSpaceKnowledgeHistoryQuery(
  spaceId: MaybeRefOrGetter<string | null | undefined>,
  nodeId: MaybeRefOrGetter<string>,
) {
  return spaceQuery(
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
  return spaceQuery(
    spaceId,
    (id) => spaceKnowledgeQueryKeys.provenance(id, targetKind, toValue(targetId)),
    (id) => listProvenance(id, targetKind, toValue(targetId)),
    () => Boolean(toValue(targetId)),
  );
}

export function useRecentlyDeletedQuery(
  spaceId: MaybeRefOrGetter<string | null>,
  enabled: MaybeRefOrGetter<boolean>,
) {
  return useQuery(
    computed(() => ({
      queryKey: spaceKnowledgeQueryKeys.deleted(toValue(spaceId)),
      queryFn: () => listRecentlyDeletedNodes(toValue(spaceId)),
      enabled: toValue(enabled),
    })),
  );
}
