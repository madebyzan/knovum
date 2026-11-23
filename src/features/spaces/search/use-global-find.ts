import { useQuery } from "@tanstack/vue-query";
import { refDebounced } from "@vueuse/core";
import { computed, shallowRef, watch, type Ref } from "vue";

import { revealContentNode, revealStructureNode } from "../canvas/structure-navigation";
import { useContextDock } from "../canvas/use-context-dock";
import { useSpaceStore } from "../session/space-store";
import { searchAcrossSpaces } from "./global-search";
import type { GlobalSearchMatch } from "./search-match";
import { useSemanticSearch } from "./use-semantic-search";

export function useGlobalFind(open: Ref<boolean>) {
  const store = useSpaceStore();
  const semanticSearch = useSemanticSearch(open, "global");
  const dock = useContextDock();
  const query = shallowRef("");
  const debouncedQuery = refDebounced(query, 240);
  const normalizedQuery = computed(() => debouncedQuery.value.trim());

  watch(open, (isOpen) => {
    if (!isOpen) query.value = "";
  });

  const searchQuery = useQuery(
    computed(() => ({
      queryKey: ["spaces", "global-search", normalizedQuery.value, semanticSearch.enabled.value],
      queryFn: ({ signal }: { signal: AbortSignal }) =>
        searchAcrossSpaces(normalizedQuery.value, {
          semantic: semanticSearch.enabled.value,
          signal,
        }),
      enabled: open.value && Boolean(normalizedQuery.value),
      staleTime: 30_000,
    })),
  );

  async function select(match: GlobalSearchMatch | undefined) {
    if (!match) return;
    open.value = false;
    if (store.activeSpaceId !== match.spaceId || store.surface.kind !== "canvas") {
      await store.openSpace(match.spaceId);
    }
    if (match.kind === "structure") {
      await revealStructureNode(match.structureNodeId);
      return;
    }
    await revealContentNode(match.nodeId);
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
    semanticSearch,
    matches: computed(() => (normalizedQuery.value ? (searchQuery.data.value ?? []) : [])),
    searching: searchQuery.isFetching,
    failed: searchQuery.isError,
    retry: searchQuery.refetch,
    select,
  };
}
