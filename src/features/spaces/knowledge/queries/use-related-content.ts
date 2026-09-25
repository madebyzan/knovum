import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, shallowRef, toValue, watch, type MaybeRefOrGetter } from "vue";

import { subscribeKnowledgeIndexStatus, type KnowledgeIndexStatus } from "../indexing/index-scheduler";
import { spaceKnowledgeQueryKeys } from "./query-options";
import { listRelatedContent } from "./related-content";

export function useRelatedContent(
  spaceId: MaybeRefOrGetter<string | null | undefined>,
  nodeId: MaybeRefOrGetter<string>,
) {
  const queryClient = useQueryClient();
  const indexStatus = shallowRef<KnowledgeIndexStatus>("idle");
  watch(
    () => toValue(spaceId),
    (id, _previous, onCleanup) => {
      if (!id) {
        indexStatus.value = "idle";
        return;
      }
      const unsubscribe = subscribeKnowledgeIndexStatus(id, (status) => {
        indexStatus.value = status;
        if (status === "ready") {
          void queryClient.invalidateQueries({ queryKey: [...spaceKnowledgeQueryKeys.space(id), "related"] });
        }
      });
      onCleanup(unsubscribe);
    },
    { immediate: true },
  );
  const query = useQuery(
    computed(() => {
      const space = toValue(spaceId);
      const node = toValue(nodeId);
      return {
        queryKey: spaceKnowledgeQueryKeys.related(space ?? "none", node),
        queryFn: () => listRelatedContent(space!, node),
        enabled: Boolean(space && node),
        staleTime: 30_000,
      };
    }),
  );
  return { query, indexStatus };
}
