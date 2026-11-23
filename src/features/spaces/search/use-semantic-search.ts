import { computed, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { textEmbeddingReady } from "../../../foundation/runtime/local-resources";
import { useLocalResourcesQuery } from "../../../foundation/runtime/use-local-resources-query";
import { isKnowledgeIndexingCancelled } from "../knowledge/indexing/indexing-outcome";
import { prepareTextEmbedding } from "../knowledge/indexing/prepare-text-embedding";
import { startKnowledgeIndexing } from "../knowledge/indexing/start-indexing";
import { useSpaceStore } from "../session/space-store";

export type SemanticSearchScope = "space" | "global";

export function useSemanticSearch(open: Ref<boolean>, scope: SemanticSearchScope = "space") {
  const store = useSpaceStore();
  const { t } = useI18n();
  const resources = useLocalResourcesQuery();
  const enabled = shallowRef(false);
  const preparing = shallowRef(false);
  const indexing = shallowRef(false);
  const available = computed(() => textEmbeddingReady(resources.data.value));

  if (scope === "space") {
    watch(
      () => store.activeSpaceId,
      () => {
        enabled.value = false;
      },
    );
  }
  watch(available, (ready) => {
    if (!ready) enabled.value = false;
  });

  function targetSpaceIds(): string[] {
    if (scope === "space") return store.activeSpaceId ? [store.activeSpaceId] : [];
    const draftId = store.draftSpace?.id;
    return store.spaces
      .filter((space) => space.archivedAt === null && space.id !== draftId)
      .map((space) => space.id);
  }

  const isCurrentScope = (expectedSpaceId: string | undefined) =>
    open.value && (scope !== "space" || store.activeSpaceId === expectedSpaceId);

  async function indexSpaces(spaceIds: readonly string[], isCurrent: () => boolean): Promise<boolean> {
    if (!spaceIds.length) return true;
    indexing.value = true;
    try {
      for (const spaceId of spaceIds) {
        if (!isCurrent()) return false;
        // oxlint-disable-next-line eslint/no-await-in-loop -- Spaces share one local embedding worker and index sequentially.
        await startKnowledgeIndexing(spaceId, t("spaces.search.semantic.indexing")).completion;
      }
      return isCurrent();
    } finally {
      indexing.value = false;
    }
  }

  async function enable() {
    const spaceIds = targetSpaceIds();
    if (preparing.value || (scope === "space" && !spaceIds.length)) return;
    const expectedSpaceId = store.activeSpaceId;
    const isCurrent = () => isCurrentScope(expectedSpaceId);
    preparing.value = true;
    try {
      if (!(await prepareTextEmbedding(t("spaces.search.semantic.preparing"))) || !isCurrent()) return;
      if (await indexSpaces(spaceIds, isCurrent)) enabled.value = true;
    } catch (error) {
      if (!isKnowledgeIndexingCancelled(error)) toast.error(getErrorMessage(error));
    } finally {
      preparing.value = false;
    }
  }

  return { enabled, preparing, indexing, enable };
}
