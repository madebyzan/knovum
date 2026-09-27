import { scheduleKnowledgeIndex } from "../knowledge/indexing/index-scheduler";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import type { NodePayloadChange } from "../model/nodes/payload-change";
import { beginSpaceMutation } from "./mutation-barrier";
import { requireActiveSpaceId, type SpaceMutationContext } from "./mutation-types";
import { replayNodeContent } from "./node-content-replay";
import {
  combineRecentSpaceHistory,
  recordSpaceHistory,
  redoSpaceHistory,
  type ReversibleSpaceChange,
  undoSpaceHistory,
} from "./stack";

export function createSpaceHistoryMutations(context: SpaceMutationContext) {
  const { nodes, history } = context;
  let canvasMutationLease: (() => void) | undefined;

  async function updateNodeContents(changes: readonly NodePayloadChange[]) {
    if (changes.length === 0) return;
    const spaceId = requireActiveSpaceId(context.activeSpaceId);
    const prepared = changes.map((change) => ({
      ...change,
      after: validateSpaceNodePayload(change.after),
    }));
    const apply = (
      entries: readonly {
        id: string;
        expectedData: NodePayloadChange["before"];
        data: NodePayloadChange["after"];
      }[],
    ) =>
      replayNodeContent({ spaceId, changes: entries, nodes, activeSpaceId: context.activeSpaceId }).then(
        (committed) => {
          scheduleKnowledgeIndex(
            spaceId,
            entries.map(({ id }) => id),
          );
          return committed;
        },
      );
    const releaseMutation = beginSpaceMutation(history);
    try {
      const committed = await apply(
        prepared.map(({ id, before, after }) => ({ id, expectedData: before, data: after })),
      );
      for (const change of prepared) {
        const committedNode = committed.find(({ id }) => id === change.id);
        if (!committedNode) throw new Error("Committed history projection is unavailable");
        change.after = committedNode.data;
      }
      recordSpaceHistory(history, {
        undo: async () => {
          await apply(prepared.map(({ id, before, after }) => ({ id, expectedData: after, data: before })));
        },
        redo: async () => {
          await apply(prepared.map(({ id, before, after }) => ({ id, expectedData: before, data: after })));
        },
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
      await updateNodeContents([{ id, before: node.data, after: { ...node.data, title: name } }]);
    },
    updateNodeContents,
    recordHistoryMutation,
    recordCanvasHistory: (entry: ReversibleSpaceChange) => recordSpaceHistory(history, entry),
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
