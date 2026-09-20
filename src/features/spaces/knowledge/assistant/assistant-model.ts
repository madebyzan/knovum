import { computed, watch } from "vue";
import type { ComputedRef, Ref } from "vue";

import { useCanvasEditorSession } from "../../canvas/editor-session";
import type { useSpaceStore } from "../../session/space-store";
import type { GroundedMessage } from "../model/grounded-message";
import type { GroundedScope } from "../model/grounded-scope";
import type { GroundedThread } from "../model/grounded-thread";
import type { KnowledgeSearchResponse } from "../model/knowledge-search-response";
import { isSpaceSemanticNode } from "../model/semantic-node";
import { activeThreadNodeIds, knowledgeScopeNodeIds, suggestedKnowledgeScope } from "./assistant-state";
import { resolveStructureKnowledgeScope } from "./structure-scope";

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
  const session = useCanvasEditorSession();
  const contentNodes = computed(() => input.store.nodes.filter(isSpaceSemanticNode));
  const structure = computed(() => session.document.value?.structure);
  const structuralScope = computed(() => {
    const documentStructure = structure.value;
    if (!documentStructure) return { selection: [], section: [] };
    return resolveStructureKnowledgeScope(
      documentStructure,
      session.selection.value.nodeIds,
      new Set(contentNodes.value.map((node) => node.id)),
    );
  });
  const selectedContentIds = computed(() => structuralScope.value.selection);
  const sectionIds = computed(() => structuralScope.value.section);
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
    const thread = input.threads.value.find((item) => item.id === value);
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
