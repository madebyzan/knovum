import { computed, shallowRef } from "vue";

import { useCanvasEditorSession } from "../canvas/editor-session";
import { currentSelectionRoots } from "../canvas/selection/selection-roots";
import { createStructureIndex } from "../model/structure/structure-index";
import { contentNodeIdsWithin } from "../model/structure/structure-queries";
import { useSpaceStore } from "../session/space-store";
import type { StartAssistantTaskInput } from "./conversation-ipc";

type Attachment = NonNullable<StartAssistantTaskInput["attachments"]>[number];

export function useConversationAttachments() {
  const canvas = useCanvasEditorSession();
  const store = useSpaceStore();
  const attachments = shallowRef<Attachment[]>([]);
  const selectedSourceNodeIds = computed(() => {
    if (!canvas.selection.value.nodeIds.length && !canvas.selection.value.structureNodeIds.length)
      return new Set<string>();
    const index = createStructureIndex(store.editSnapshot.structure);
    const roots = currentSelectionRoots(
      canvas.selection.value.nodeIds,
      canvas.selection.value.structureNodeIds,
      index,
    );
    return new Set(
      contentNodeIdsWithin(
        roots.map(({ id }) => id),
        index,
      ),
    );
  });
  const visibleAttachments = computed(() =>
    attachments.value.map(({ nodeId }) => ({
      nodeId,
      title: store.nodes.find(({ id }) => id === nodeId)?.data.title ?? nodeId,
    })),
  );
  const attachmentCandidates = computed(() => {
    const attachedIds = new Set(attachments.value.map(({ nodeId }) => nodeId));
    return store.nodes
      .filter((node) => Boolean(node.data.knowledgeRevision ?? node.data.sourceRevision))
      .map((node) => ({
        nodeId: node.id,
        title: node.data.title,
        selected: selectedSourceNodeIds.value.has(node.id),
        attached: attachedIds.has(node.id),
      }));
  });

  function addSelectedAttachments() {
    addNodeAttachments([...selectedSourceNodeIds.value]);
  }

  function addNodeAttachments(nodeIds: readonly string[]) {
    const selected = new Set(nodeIds);
    const next = new Map(attachments.value.map((item) => [item.nodeId, item]));
    for (const node of store.nodes) {
      if (!selected.has(node.id)) continue;
      const revision = node.data.knowledgeRevision ?? node.data.sourceRevision;
      if (revision) next.set(node.id, { nodeId: node.id, sourceRevision: revision, role: "source" });
    }
    attachments.value = [...next.values()];
  }

  function removeAttachment(nodeId: string) {
    attachments.value = attachments.value.filter((item) => item.nodeId !== nodeId);
  }

  return {
    attachments,
    visibleAttachments,
    attachmentCandidates,
    addSelectedAttachments,
    addNodeAttachments,
    removeAttachment,
  };
}
