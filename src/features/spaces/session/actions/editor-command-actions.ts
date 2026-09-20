import type { ExecuteSpaceEditRequest, ReplaySpaceEditRequest } from "../../canvas/contract/editor-command";
import type { SpaceEditResult } from "../../canvas/contract/editor-result";
import { createSpaceNode } from "../../canvas/node";
import { executeSpaceEdit, replaySpaceEdit } from "../../canvas/persistence/editor-command-repository";
import { scheduleKnowledgeIndex } from "../../knowledge/indexing/index-scheduler";
import type { SpaceState } from "../session-state";

export function createEditorCommandActions(state: SpaceState) {
  async function submit(operation: (spaceId: string) => Promise<SpaceEditResult>) {
    const spaceId = state.activeSpaceId.value;
    if (!spaceId) throw new Error("An active Space is required for editing");
    const result = await operation(spaceId);
    if (state.activeSpaceId.value === spaceId && result.content) {
      const previous = new Map(state.nodes.value.map((node) => [node.id, node]));
      state.nodes.value = result.content.nodes.map((record) => {
        const node = createSpaceNode(record.data, { x: record.initialX, y: record.initialY }, record.id);
        const old = previous.get(record.id);
        return old?.hidden === undefined ? node : { ...node, hidden: old.hidden };
      });
      state.relations.value = result.content.relations;
      const added = result.content.nodes.filter((node) => !previous.has(node.id)).map((node) => node.id);
      if (added.length) scheduleKnowledgeIndex(spaceId, added);
    }
    return result;
  }
  return {
    executeEditorCommand: (request: ExecuteSpaceEditRequest) =>
      submit((spaceId) => executeSpaceEdit(spaceId, request)),
    replayEditorCommand: (request: ReplaySpaceEditRequest) =>
      submit((spaceId) => replaySpaceEdit(spaceId, request)),
  };
}
