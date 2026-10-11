import { useQuery } from "@tanstack/vue-query";
import { refDebounced } from "@vueuse/core";
import { take } from "es-toolkit";
import { computed, ref, watch, type Ref } from "vue";

import { revealContentNode, revealStructureNode } from "../canvas/structure-navigation";
import { useContextDock } from "../canvas/use-context-dock";
import { useSpaceStore } from "../session/space-store";
import { useSemanticSearch } from "./use-semantic-search";

export interface StructureTitleMatch {
  kind: "structure";
  structureNodeId: string;
  nodeKind: "group" | "section";
  title: string;
}

export function useFind(open: Ref<boolean>) {
  const store = useSpaceStore();
  const semanticSearch = useSemanticSearch(open);
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

  async function select(value: unknown) {
    if (typeof value !== "string") return;
    if (value.startsWith("structure:")) {
      const match = structureMatches.value.find(
        ({ structureNodeId }) => structureNodeId === value.slice("structure:".length),
      );
      if (!match) return;
      open.value = false;
      await revealStructureNode(match.structureNodeId);
      return;
    }
    if (value.startsWith("match:")) {
      const match = matches.value.find(({ chunkId }) => chunkId === value.slice("match:".length));
      if (!match) return;
      open.value = false;
      await revealContentNode(match.nodeId);
      dock.openSource({
        nodeId: match.nodeId,
        nodeTitle: match.nodeTitle,
        chunkId: match.chunkId,
        locator: match.locator,
        text: match.text,
      });
      return;
    }
    if (value.startsWith("node:")) {
      const node = defaultNodes.value.find(({ id }) => id === value.slice("node:".length));
      if (!node) return;
      open.value = false;
      await revealContentNode(node.id);
    }
  }

  return {
    query,
    semanticSearch,
    matches,
    structureMatches,
    semantic,
    searching: searchQuery.isFetching,
    searchFailed: searchQuery.isError,
    defaultNodes,
    select,
    retry: searchQuery.refetch,
  };
}
