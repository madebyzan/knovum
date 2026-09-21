import { computed, watch } from "vue";
import type { ComputedRef, Ref } from "vue";

import { useCanvasSelectionProjection } from "../../canvas/editor-session";
import { createStructureIndex, effectiveOperationRoots } from "../../model/structure/structure-index";
import { contentNodeIdsWithin, sectionAncestorOf } from "../../model/structure/structure-queries";
import type { useSpaceStore } from "../../session/space-store";
import type { GroundedMessage } from "../model/grounded-message";
import type { GroundedScope } from "../model/grounded-scope";
import type { GroundedThread } from "../model/grounded-thread";
import type { KnowledgeSearchResponse } from "../model/knowledge-search-response";
import { isSpaceSemanticNode } from "../model/semantic-node";
import { activeThreadNodeIds, knowledgeScopeNodeIds, suggestedKnowledgeScope } from "./assistant-state";

type SpaceStore = ReturnType<typeof useSpaceStore>;

interface KnowledgeAssistantModelInput {
  store: SpaceStore;
  threads: ComputedRef<GroundedThread[]>;
  messages: ComputedRef<GroundedMessage[]>;
  threadId: Ref<string | undefined>;
  scope: Ref<GroundedScope>;
  semantic: Ref<KnowledgeSearchResponse["semantic"]>;
  generation: Ref<number>;
}

export function useKnowledgeAssistantModel(input: KnowledgeAssistantModelInput) {
  const contentNodes = computed(() => input.store.nodes.filter(isSpaceSemanticNode));
  const { selection } = useCanvasSelectionProjection(() => input.store.nodes);
  const structureIndex = computed(() => createStructureIndex(input.store.editSnapshot.structure));
  const selectedStructureNodeIds = computed(() => {
    const contentStructureNodeIds = selection.value.nodeIds.flatMap((nodeId) => {
      const structureNodeId = structureIndex.value.nodeIdToStructureNodeId.get(nodeId);
      return structureNodeId ? [structureNodeId] : [];
    });
    return effectiveOperationRoots(
      [...selection.value.structureNodeIds, ...contentStructureNodeIds],
      structureIndex.value,
    ).map(({ id }) => id);
  });
  const selectedContentIds = computed(() => {
    const eligible = new Set(contentNodes.value.map(({ id }) => id));
    return contentNodeIdsWithin(selectedStructureNodeIds.value, structureIndex.value).filter((id) =>
      eligible.has(id),
    );
  });
  const sectionId = computed(() => {
    if (selectedStructureNodeIds.value.length !== 1) return undefined;
    const structureNodeId = selectedStructureNodeIds.value[0]!;
    const structureNode = structureIndex.value.structureById.get(structureNodeId);
    return structureNode?.kind === "section"
      ? structureNode.id
      : sectionAncestorOf(structureNodeId, structureIndex.value)?.id;
  });
  const sectionIds = computed(() =>
    sectionId.value ? contentNodeIdsWithin([sectionId.value], structureIndex.value) : [],
  );
  const activeThread = computed(() =>
    input.threads.value.find((thread) => thread.id === input.threadId.value),
  );
  const draftIds = computed(() =>
    knowledgeScopeNodeIds(input.scope.value, selectedContentIds.value, sectionIds.value),
  );
  const activeIds = computed(() => activeThreadNodeIds(activeThread.value, draftIds.value));
  const activeScope = computed<GroundedScope>(() => activeThread.value?.scope ?? input.scope.value);
  const scopeCounts = computed(() => ({
    selection:
      activeThread.value?.scope === "selection" ? activeIds.value.length : selectedContentIds.value.length,
    section: activeThread.value?.scope === "section" ? activeIds.value.length : sectionIds.value.length,
    space: contentNodes.value.length,
  }));
  const scopeTitles = computed(() =>
    activeScope.value === "selection"
      ? activeIds.value
          .map((id) => input.store.nodes.find((node) => node.id === id)?.data.title)
          .filter((title): title is string => Boolean(title))
      : [],
  );

  function resetThread() {
    input.generation.value += 1;
    input.threadId.value = undefined;
    input.semantic.value = "empty";
    input.scope.value = suggestedKnowledgeScope(selectedContentIds.value, sectionIds.value);
  }
  function selectThread(value: string) {
    if (value === "new") return resetThread();
    const thread = input.threads.value.find((candidate) => candidate.id === value);
    if (!thread) return;
    input.generation.value += 1;
    input.threadId.value = thread.id;
    input.scope.value = thread.scope;
    input.semantic.value = "empty";
  }

  watch([selectedContentIds, sectionIds], () => {
    if (!input.threadId.value && input.messages.value.length === 0) {
      input.scope.value = suggestedKnowledgeScope(selectedContentIds.value, sectionIds.value);
    }
  });
  watch(() => input.store.activeSpace?.id, resetThread, { immediate: true });

  return {
    contentNodes,
    selectedContentIds,
    sectionIds,
    activeThread,
    activeIds,
    activeScope,
    scopeCounts,
    scopeTitles,
    resetThread,
    selectThread,
  };
}
