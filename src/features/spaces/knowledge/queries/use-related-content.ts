import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, shallowRef, toValue, watch, type MaybeRefOrGetter } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { decodeAppError } from "../../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { startResourcePreparation } from "../../../../foundation/runtime/resource-preparation";
import {
  scheduleSpaceKnowledgeIndex,
  subscribeKnowledgeIndexStatus,
  type KnowledgeIndexStatus,
} from "../indexing/index-scheduler";
import { SPACE_KNOWLEDGE_QUERY_KEYS } from "./query-options";
import { listRelatedContent } from "./related-content";

export function useRelatedContent(
  spaceId: MaybeRefOrGetter<string | null | undefined>,
  nodeId: MaybeRefOrGetter<string>,
) {
  const queryClient = useQueryClient();
  const { t } = useI18n();
  const preparing = shallowRef(false);
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
          void queryClient.invalidateQueries({
            queryKey: [...SPACE_KNOWLEDGE_QUERY_KEYS.space(id), "related"],
          });
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
        queryKey: SPACE_KNOWLEDGE_QUERY_KEYS.related(space ?? "none", node),
        queryFn: () => listRelatedContent(space!, node),
        enabled: Boolean(space && node),
        staleTime: 30_000,
      };
    }),
  );
  async function prepare() {
    const space = toValue(spaceId);
    if (!space || preparing.value) return;
    preparing.value = true;
    try {
      await startResourcePreparation({ kind: "textEmbedding" }, t("spaces.search.semantic.preparing"))
        .completion;
      if (toValue(spaceId) === space) scheduleSpaceKnowledgeIndex(space);
    } catch (error) {
      if (decodeAppError(error)?.code !== "local_resource_preparation_cancelled") {
        toast.error(getErrorMessage(error));
      }
    } finally {
      preparing.value = false;
    }
  }

  return { query, indexStatus, preparing, prepare };
}
