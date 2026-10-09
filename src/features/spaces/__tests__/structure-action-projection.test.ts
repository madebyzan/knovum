import { beforeEach, expect, it, vi } from "vitest";
import { shallowRef } from "vue";

import { createSpaceNode } from "../canvas/node";
import { createSpaceTextPayload } from "../editor/tiptap-content";
import { createSpaceHistoryState } from "../history/history-state";
import { redoSpaceHistory, undoSpaceHistory } from "../history/stack";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { createStructureActions } from "../session/actions/structure-actions";
import type { loadSpaceSessionProjection } from "../session/space-projection";
import type { spaceEditOperations } from "../structure/space-edit-operations";

const operations = vi.hoisted(() => ({
  execute: vi.fn<typeof spaceEditOperations.execute>(),
  load: vi.fn<typeof spaceEditOperations.load>(),
  undo: vi.fn<typeof spaceEditOperations.undo>(),
  redo: vi.fn<typeof spaceEditOperations.redo>(),
}));
const fullProjection = vi.hoisted(() => vi.fn<typeof loadSpaceSessionProjection>());
vi.mock("../structure/space-edit-operations", () => ({ spaceEditOperations: operations }));
vi.mock("../session/space-projection", () => ({ loadSpaceSessionProjection: fullProjection }));

function setup() {
  const { history } = createSpaceHistoryState();
  const node = createSpaceNode(createSpaceTextPayload("Text"), { x: 0, y: 0 }, "text");
  const context = {
    history,
    activeSpaceId: shallowRef<string | undefined>("space"),
    nodes: shallowRef([node]),
    relations: shallowRef([]),
    editSnapshot: shallowRef<SpaceEditSnapshot>({
      editRevision: 0,
      structure: [
        { id: "content", kind: "content", nodeId: "text", parentId: null, order: 0, locked: false },
      ],
      layouts: [
        {
          kind: "content",
          structureNodeId: "content",
          transform: { x: 0, y: 0, scale: 1, rotation: 0 },
          size: { width: 100, height: 40 },
          sizingMode: "frame",
        },
      ],
      relationStyles: [],
    }),
    ensureActiveSpacePersisted: async () => undefined,
  };
  return { context, actions: createStructureActions(context) };
}

beforeEach(() => {
  vi.resetAllMocks();
  operations.execute.mockResolvedValue({
    mutationId: "mutation",
    appliedAtRevision: 1,
    selectionSuggestion: [],
  });
  operations.undo.mockResolvedValue({ appliedAtRevision: 2, selectionSuggestion: [] });
  operations.redo.mockResolvedValue({ appliedAtRevision: 3, selectionSuggestion: [] });
});

it("reloads only the authoritative layout snapshot after movement and its history transitions", async () => {
  const { context, actions } = setup();
  const nodes = context.nodes.value;
  const structure = context.editSnapshot.value.structure;
  const next = { ...structuredClone(context.editSnapshot.value), editRevision: 1 };
  operations.load.mockResolvedValue(next);
  fullProjection.mockResolvedValue({ editSnapshot: next, nodes: [], relations: [], scenes: [] });
  await actions.executeSpaceEdit({ kind: "structure.move", rootIds: ["content"], delta: { x: 10, y: 0 } });
  expect(context.nodes.value).toBe(nodes);
  expect(context.editSnapshot.value.structure).toBe(structure);
  expect(context.editSnapshot.value.editRevision).toBe(1);
  await undoSpaceHistory(context.history);
  await redoSpaceHistory(context.history);
  expect(context.nodes.value).toBe(nodes);
  expect(fullProjection).not.toHaveBeenCalled();
  expect(operations.load).toHaveBeenCalledTimes(3);
});

it("refreshes content membership after deletion and undo", async () => {
  const { context, actions } = setup();
  const restored = {
    nodes: context.nodes.value,
    editSnapshot: context.editSnapshot.value,
    relations: [],
    scenes: [],
  };
  fullProjection
    .mockResolvedValueOnce({
      nodes: [],
      relations: [],
      scenes: [],
      editSnapshot: { editRevision: 1, structure: [], layouts: [], relationStyles: [] },
    })
    .mockResolvedValueOnce(restored);
  await actions.executeSpaceEdit({ kind: "structure.delete-subtrees", rootIds: ["content"] });
  expect(context.nodes.value).toEqual([]);
  await undoSpaceHistory(context.history);
  expect(context.nodes.value[0]?.id).toBe("text");
  expect(operations.load).not.toHaveBeenCalled();
});
