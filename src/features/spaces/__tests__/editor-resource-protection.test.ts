import { beforeEach, expect, it, vi } from "vitest";

import { createAppError } from "../../../foundation/errors/app-error";
import { undoSpaceHistory, redoSpaceHistory } from "../history/stack";
import { createStructureActions } from "../session/actions/structure-actions";
import { createStructureTransferActions } from "../session/actions/structure-transfer-actions";
import type { loadSpaceSessionProjection } from "../session/space-projection";
import type { spaceEditOperations } from "../structure/space-edit-operations";
import type { moveStructureSelectionBetweenSpaces } from "../structure/structure-transfer-operations";
import {
  EditorResourceProtectedError,
  editorsWithinStructure,
} from "../workspace/editor-resource-protection";
import { removeSpaceWorkItems } from "../workspace/work-items";
import { activateTestEditor, createTestSessionContext } from "./support/session-context";

const mocks = vi.hoisted(() => ({
  execute: vi.fn<typeof spaceEditOperations.execute>(),
  undo: vi.fn<typeof spaceEditOperations.undo>(),
  redo: vi.fn<typeof spaceEditOperations.redo>(),
  load: vi.fn<typeof spaceEditOperations.load>(),
  move: vi.fn<typeof moveStructureSelectionBetweenSpaces>(),
  projection: vi.fn<typeof loadSpaceSessionProjection>(),
}));
vi.mock("../structure/space-edit-operations", () => ({ spaceEditOperations: mocks }));
vi.mock("../structure/structure-transfer-operations", () => ({
  moveStructureSelectionBetweenSpaces: mocks.move,
}));
vi.mock("../session/space-projection", () => ({ loadSpaceSessionProjection: mocks.projection }));
beforeEach(() => {
  vi.resetAllMocks();
  mocks.execute.mockResolvedValue({ mutationId: "mutation", appliedAtRevision: 1, selectionSuggestion: [] });
});

function setup() {
  const context = createTestSessionContext();
  context.editSnapshot.value.structure = [
    { id: "group", kind: "group", title: "Group", parentId: null, order: 0, locked: false },
    { id: "content", kind: "content", nodeId: "node", parentId: "group", order: 0, locked: false },
    { id: "other", kind: "content", nodeId: "other", parentId: null, order: 1, locked: false },
  ];
  const tab = activateTestEditor(context, "document", "node");
  const video = activateTestEditor(context, "video", "video");
  const actions = createStructureActions({ ...context, ensureActiveSpacePersisted: async () => undefined });
  const transfer = createStructureTransferActions({
    ...context,
    recordCommittedSpaceEdit: actions.recordCommittedSpaceEdit,
  });
  return { context, tab, video, actions, transfer };
}

it.each(["content", "group"])(
  "blocks deletion of %s while its editor is open, including mixed selections",
  async (root) => {
    const { context, actions } = setup();
    await expect(
      actions.executeSpaceEdit({ kind: "structure.delete-subtrees", rootIds: [root, "other"] }),
    ).rejects.toBeInstanceOf(EditorResourceProtectedError);
    expect(mocks.execute).not.toHaveBeenCalled();
    expect(context.editorTabs.value).toHaveLength(2);
    expect(context.history.pendingMutations).toBe(0);
  },
);

it("blocks cross-Space transfer without committing or closing the editor", async () => {
  const { context, transfer } = setup();
  await expect(transfer.moveStructureNodes(["group"], "target")).rejects.toBeInstanceOf(
    EditorResourceProtectedError,
  );
  expect(mocks.move).not.toHaveBeenCalled();
  expect(context.editorTabs.value).toHaveLength(2);
});

it("allows layout changes and passes every Space's live editor identities to Rust", async () => {
  const { context, actions, tab, video } = setup();
  context.editorTabs.value = [
    ...context.editorTabs.value,
    { id: "foreign", kind: "document", spaceId: "second", nodeId: "foreign-node" },
  ];
  mocks.load.mockResolvedValue(context.editSnapshot.value);
  await actions.executeSpaceEdit({ kind: "structure.set-lock", rootIds: ["group"], locked: true });
  expect(mocks.execute).toHaveBeenCalledWith(expect.anything(), [
    { spaceId: tab.spaceId, nodeId: tab.nodeId },
    { spaceId: video.spaceId, nodeId: video.nodeId },
    { spaceId: "second", nodeId: "foreign-node" },
  ]);
  expect(context.editorTabs.value).toHaveLength(3);
});

it.each(["undo", "redo"] as const)(
  "retains the %s entry when Rust rejects a protected resource change",
  async (direction) => {
    const { context, actions, tab, video } = setup();
    actions.recordCommittedSpaceEdit();
    if (direction === "redo") {
      context.history.redoStack = context.history.undoStack;
      context.history.undoStack = [];
    }
    const before = [...context.history[`${direction}Stack`]];
    mocks[direction].mockRejectedValue(createAppError("space_editor_resource_protected", "Protected"));
    await expect(
      (direction === "undo" ? undoSpaceHistory : redoSpaceHistory)(context.history),
    ).rejects.toThrow("Protected");
    expect(context.history[`${direction}Stack`]).toEqual(before);
    expect(mocks[direction]).toHaveBeenCalledWith("inbox", 0, [
      { spaceId: tab.spaceId, nodeId: tab.nodeId },
      { spaceId: video.spaceId, nodeId: video.nodeId },
    ]);
    expect(context.editorTabs.value).toHaveLength(2);
  },
);

it("releases protection only after the editor tab is removed", async () => {
  const { context, tab, actions } = setup();
  expect(
    editorsWithinStructure(context.editorTabs.value, "inbox", context.editSnapshot.value, ["group"]),
  ).toEqual([tab]);
  removeSpaceWorkItems(context, new Set([tab.id]));
  mocks.projection.mockResolvedValue({
    nodes: [],
    scenes: [],
    relations: [],
    editSnapshot: context.editSnapshot.value,
  });
  await actions.executeSpaceEdit({ kind: "structure.delete-subtrees", rootIds: ["group"] });
  expect(mocks.execute).toHaveBeenCalledOnce();
  expect(context.editorTabs.value).toHaveLength(1);
});
