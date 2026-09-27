import { useQuery } from "@tanstack/vue-query";
import { refDebounced } from "@vueuse/core";
import { take } from "es-toolkit";
import { computed, ref, watch, type Ref } from "vue";

import { useCanvasEditorSession } from "../canvas/editor-session";
import { revealContentNode, revealStructureNode } from "../canvas/structure-navigation";
import { useContextDock } from "../canvas/use-context-dock";
import type { KnowledgeSearchMatch } from "../knowledge/model/schema/search-match";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { useSpaceStore } from "../session/space-store";
import { useSemanticSearch } from "./use-semantic-search";

export interface StructureTitleMatch {
  kind: "structure";
  structureNodeId: string;
  nodeKind: "group" | "section";
  title: string;
}

export function useFind(open: Ref<boolean>, close: () => void) {
  const store = useSpaceStore();
  const semanticSearch = useSemanticSearch(open);
  const editorSession = useCanvasEditorSession();
  const dock = useContextDock();
  const query = ref("");
  const debouncedQuery = refDebounced(query, 300);

  const defaultNodes = computed(() => take(store.nodes, 10));

  watch(open, (value) => {
    if (value) query.value = "";
  });

  const trimmed = computed(() => debouncedQuery.value.trim());

  const searchQuery = useQuery(
    computed(() => ({
      queryKey: ["spaces", store.activeSpaceId, "find", trimmed.value, semanticSearch.enabled.value],
      queryFn: ({ signal }: { signal: AbortSignal }) =>
        store.search(trimmed.value, semanticSearch.enabled.value, signal),
      enabled: open.value && Boolean(trimmed.value),
      staleTime: 30_000,
    })),
  );

  const matches = computed(() => (trimmed.value ? (searchQuery.data.value?.matches ?? []) : []));
  const structureMatches = computed<StructureTitleMatch[]>(() => {
    if (!trimmed.value) return [];
    const term = trimmed.value.toLocaleLowerCase();
    return store.editSnapshot.structure
      .flatMap((structureNode) =>
        structureNode.kind !== "content" && structureNode.title.toLocaleLowerCase().includes(term)
          ? [
              {
                kind: "structure" as const,
                structureNodeId: structureNode.id,
                nodeKind: structureNode.kind,
                title: structureNode.title,
              },
            ]
          : [],
      )
      .slice(0, 20);
  });
  const semantic = computed(() => searchQuery.data.value?.semantic ?? "empty");
  const searching = computed(() => searchQuery.isFetching.value);
  const searchFailed = computed(() => searchQuery.isError.value);

  async function select(value: unknown) {
    if (!value) return;
    close();
    if (isStructureTitleMatch(value)) {
      await revealStructureNode(store, editorSession, value.structureNodeId);
      return;
    }
    if (query.value.trim()) {
      const match = value as KnowledgeSearchMatch;
      await revealContentNode(store, editorSession, match.nodeId);
      dock.openSource({
        nodeId: match.nodeId,
        nodeTitle: match.nodeTitle,
        chunkId: match.chunkId,
        locator: match.locator,
        text: match.text,
      });
    } else await revealContentNode(store, editorSession, (value as SpaceCanvasNode).id);
  }

  return {
    query,
    semanticSearch,
    matches,
    structureMatches,
    semantic,
    searching,
    searchFailed,
    defaultNodes,
    select,
    retry: () => searchQuery.refetch(),
  };
}

function isStructureTitleMatch(value: unknown): value is StructureTitleMatch {
  return typeof value === "object" && value !== null && "kind" in value && value.kind === "structure";
}
