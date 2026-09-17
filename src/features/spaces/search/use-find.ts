import { useQuery } from "@tanstack/vue-query";
import { refDebounced } from "@vueuse/core";
import { take } from "es-toolkit";
import { computed, ref, watch, type Ref } from "vue";

import { useCanvasEditorSession } from "../canvas/editor-session";
import { useContextDock } from "../canvas/use-context-dock";
import type { KnowledgeSearchMatch } from "../knowledge/model/knowledge-search-match";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { useSpaceStore } from "../session/space-store";

export function useFind(open: Ref<boolean>, close: () => void) {
  const store = useSpaceStore();
  const editorSession = useCanvasEditorSession();
  const dock = useContextDock();
  const query = ref("");
  const debouncedQuery = refDebounced(query, 300);

  const defaultNodes = computed(() =>
    take(
      store.nodes.filter((node) => node.data.kind !== "section"),
      10,
    ),
  );

  watch(open, (value) => {
    if (value) query.value = "";
  });

  const trimmed = computed(() => debouncedQuery.value.trim());

  const searchQuery = useQuery(
    computed(() => ({
      queryKey: ["spaces", store.activeSpaceId, "find", trimmed.value],
      queryFn: () => store.search(trimmed.value),
      enabled: Boolean(trimmed.value),
      staleTime: 30_000,
    })),
  );

  const matches = computed(() => (trimmed.value ? (searchQuery.data.value?.matches ?? []) : []));
  const semantic = computed(() => searchQuery.data.value?.semantic ?? "empty");
  const searching = computed(() => searchQuery.isFetching.value);
  const searchFailed = computed(() => searchQuery.isError.value);

  function select(value: unknown) {
    if (!value) return;
    close();
    if (query.value.trim()) {
      const match = value as KnowledgeSearchMatch;
      editorSession.reveal([match.nodeId], "focus");
      dock.openSource({
        nodeId: match.nodeId,
        nodeTitle: match.nodeTitle,
        chunkId: match.chunkId,
        locator: match.locator,
        text: match.text,
      });
    } else editorSession.reveal([(value as SpaceCanvasNode).id], "focus");
  }

  return {
    query,
    matches,
    semantic,
    searching,
    searchFailed,
    defaultNodes,
    select,
    retry: () => searchQuery.refetch(),
  };
}
