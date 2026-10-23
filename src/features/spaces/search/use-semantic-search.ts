import { computed, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { textEmbeddingReady } from "../../../foundation/runtime/local-resources";
import { startResourcePreparation } from "../../../foundation/runtime/resource-preparation";
import { useLocalResourcesQuery } from "../../../foundation/runtime/use-local-resources-query";
import { isKnowledgeIndexingCancelled, startKnowledgeIndexing } from "../knowledge/indexing/start-indexing";
import { useSpaceStore } from "../session/space-store";

export function useSemanticSearch(open: Ref<boolean>) {
  const store = useSpaceStore();
  const { t } = useI18n();
  const resources = useLocalResourcesQuery();
  const enabled = shallowRef(false);
  const preparing = shallowRef(false);
  const indexing = shallowRef(false);
  const available = computed(() => textEmbeddingReady(resources.data.value));
  watch(
    () => store.activeSpaceId,
    () => {
      enabled.value = false;
    },
  );
  watch(available, (ready) => {
    if (!ready) enabled.value = false;
  });

  async function enable() {
    const spaceId = store.activeSpaceId;
    if (!spaceId || preparing.value) return;
    preparing.value = true;
    try {
      await startResourcePreparation({ kind: "textEmbedding" }, t("spaces.search.semantic.preparing"))
        .completion;
      if (!open.value || store.activeSpaceId !== spaceId) return;
      indexing.value = true;
      try {
        await startKnowledgeIndexing(spaceId, t("spaces.search.semantic.indexing")).completion;
        if (open.value && store.activeSpaceId === spaceId) enabled.value = true;
      } finally {
        indexing.value = false;
      }
    } catch (error) {
      if (!isKnowledgeIndexingCancelled(error)) reportSemanticSearchError(error);
    } finally {
      preparing.value = false;
    }
  }

  return { enabled, preparing, indexing, enable };
}

function reportSemanticSearchError(error: unknown) {
  if (decodeAppError(error)?.code !== "local_resource_preparation_cancelled") {
    toast.error(getErrorMessage(error));
  }
}
