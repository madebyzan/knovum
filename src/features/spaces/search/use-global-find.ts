import { useQuery } from "@tanstack/vue-query";
import { refDebounced } from "@vueuse/core";
import { computed, shallowRef, watch, type Ref } from "vue";

import { useCanvasEditorSession } from "../canvas/editor-session";
import { useContextDock } from "../canvas/use-context-dock";
import { useSpaceStore } from "../session/space-store";
import { searchKnowledgeAcrossSpaces, type GlobalKnowledgeSearchMatch } from "./global-search-repository";

export function useGlobalFind(open: Ref<boolean>) {
  const store = useSpaceStore();
  const editorSession = useCanvasEditorSession();
  const dock = useContextDock();
  const query = shallowRef("");
  const debouncedQuery = refDebounced(query, 240);
  const normalizedQuery = computed(() => debouncedQuery.value.trim());

  watch(open, (isOpen) => {
    if (!isOpen) query.value = "";
  });

  const result = useQuery(
    computed(() => ({
      queryKey: ["spaces", "global-search", normalizedQuery.value],
      queryFn: () => searchKnowledgeAcrossSpaces(normalizedQuery.value),
      enabled: Boolean(normalizedQuery.value),
      staleTime: 30_000,
    })),
  );

  async function select(match: GlobalKnowledgeSearchMatch | undefined) {
    if (!match) return;
    open.value = false;
    if (store.activeSpaceId !== match.spaceId) await store.openSpace(match.spaceId);
    editorSession.reveal([match.nodeId], "focus");
    dock.openSource({
      nodeId: match.nodeId,
      nodeTitle: match.nodeTitle,
      chunkId: match.chunkId,
      locator: match.locator,
      text: match.text,
    });
  }

  return {
    query,
    matches: computed(() => (normalizedQuery.value ? (result.data.value ?? []) : [])),
    searching: computed(() => result.isFetching.value),
    failed: computed(() => result.isError.value),
    retry: result.refetch,
    select,
  };
}
