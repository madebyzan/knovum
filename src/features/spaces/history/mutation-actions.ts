import { scheduleKnowledgeIndex } from "../knowledge/indexing/index-scheduler";
import { prepareSpaceKnowledgePayload } from "../knowledge/model/node-knowledge";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { applyData } from "./mutation-application";
import { beginSpaceMutation } from "./mutation-barrier";
import { requireActiveSpaceId, type SpaceMutationContext, type NodeDataChange } from "./mutation-types";
import {
  combineRecentSpaceHistory,
  recordSpaceHistory,
  redoSpaceHistory,
  type SpaceHistoryEntry,
  undoSpaceHistory,
} from "./stack";

export function createSpaceHistoryMutations(context: SpaceMutationContext) {
  const { nodes, history } = context;
  let canvasMutationLease: (() => void) | undefined;

  async function updateManyData(changes: readonly NodeDataChange[]) {
    if (changes.length === 0) return;
    const spaceId = requireActiveSpaceId(context.activeSpaceId);
    const prepared = changes.map((change) => ({
      ...change,
      after: prepareSpaceKnowledgePayload(validateSpaceNodePayload(change.after)),
    }));
    const apply = (entries: readonly { id: string; expectedData: NodeDataChange["before"]; data: NodeDataChange["after"] }[]) =>
      applyData({ spaceId, changes: entries, nodes, activeSpaceId: context.activeSpaceId }).then(() =>
        scheduleKnowledgeIndex(
          spaceId,
          entries.map(({ id }) => id),
        ),
      );
    const releaseMutation = beginSpaceMutation(history);
    try {
      await apply(prepared.map(({ id, before, after }) => ({ id, expectedData: before, data: after })));
      recordSpaceHistory(history, {
        undo: () => apply(prepared.map(({ id, before, after }) => ({ id, expectedData: after, data: before }))),
        redo: () => apply(prepared.map(({ id, before, after }) => ({ id, expectedData: before, data: after }))),
      });
    } finally {
      releaseMutation();
    }
  }

  async function recordHistoryMutation(action: () => Promise<void>) {
    const startIndex = history.undoStack.length;
    const releaseMutation = beginSpaceMutation(history);
    try {
      await action();
    } finally {
      combineRecentSpaceHistory(history, startIndex);
      releaseMutation();
    }
  }

  return {
    renameNode: async (id: string, title: string) => {
      const node = nodes.value.find((candidate) => candidate.id === id);
      if (!node) throw new Error("Cannot rename an unavailable Space node");
      const name = title.trim();
      if (!name) throw new Error("Node title cannot be empty");
      if (name === node.data.title) return;
      await updateManyData([{ id, before: node.data, after: { ...node.data, title: name } }]);
    },
    updateManyData,
    recordHistoryMutation,
    recordCanvasHistory: (entry: SpaceHistoryEntry) => recordSpaceHistory(history, entry),
    setCanvasMutationPending: (pending: boolean) => {
      if (pending) canvasMutationLease = beginSpaceMutation(history);
      else {
        canvasMutationLease?.();
        canvasMutationLease = undefined;
      }
    },
    undo: () => undoSpaceHistory(history),
    redo: () => redoSpaceHistory(history),
  };
}
