import { computed, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { textEmbeddingReady } from "../../../foundation/runtime/local-resources";
import { prepareModelCapability } from "../../../foundation/runtime/model-preparation";
import { ResourcePreparationCancelledError } from "../../../foundation/runtime/resource-preparation";
import { useLocalResourcesQuery } from "../../../foundation/runtime/resource-query";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { indexKnowledgeRetrieval } from "../knowledge/persistence/retrieval-index";
import { useSpaceStore } from "../session/space-store";

export function useSemanticSearch(open: Ref<boolean>) {
  const store = useSpaceStore();
  const { t } = useI18n();
  const resources = useLocalResourcesQuery();
  const tasks = useTaskStore();
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
      await prepareModelCapability("textEmbedding", t("spaces.search.semantic.preparing"));
      if (!open.value || store.activeSpaceId !== spaceId) return;
      indexing.value = true;
      const id = tasks.start(t("spaces.search.semantic.indexing"));
      try {
        await indexKnowledgeRetrieval(spaceId);
        tasks.succeed(id);
        if (open.value && store.activeSpaceId === spaceId) enabled.value = true;
      } catch (error) {
        tasks.fail(id, String(error));
        throw error;
      } finally {
        indexing.value = false;
      }
    } catch (error) {
      if (error instanceof ResourcePreparationCancelledError) return;
      toast.error(String(error));
    } finally {
      preparing.value = false;
    }
  }

  return { enabled, preparing, indexing, enable };
}
