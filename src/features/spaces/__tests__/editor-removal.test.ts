import { beforeEach, expect, it, onTestFinished, vi } from "vitest";

import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { createStructureActions } from "../session/actions/structure-actions";
import { createStructureTransferActions } from "../session/actions/structure-transfer-actions";
import type { loadSpaceSessionProjection } from "../session/space-projection";
import type { spaceEditOperations } from "../structure/space-edit-operations";
import type { moveStructureSelectionBetweenSpaces } from "../structure/structure-transfer-operations";
import { prepareEditorRemoval } from "../workspace/editor-removal";
import { registerSpaceSessionParticipant } from "../workspace/session-participants";
import { activateTestEditor, createTestSessionContext } from "./support/session-context";

const mocks = vi.hoisted(() => ({
  execute: vi.fn<typeof spaceEditOperations.execute>(),
  load: vi.fn<typeof spaceEditOperations.load>(),
  move: vi.fn<typeof moveStructureSelectionBetweenSpaces>(),
  projection: vi.fn<typeof loadSpaceSessionProjection>(),
}));
vi.mock("../structure/space-edit-operations", () => ({ spaceEditOperations: mocks }));
vi.mock("../structure/structure-transfer-operations", () => ({
  moveStructureSelectionBetweenSpaces: mocks.move,
}));
vi.mock("../session/space-projection", () => ({ loadSpaceSessionProjection: mocks.projection }));
beforeEach(() => vi.resetAllMocks());

function setup() {
  const context = createTestSessionContext();
  context.editSnapshot.value.structure = [
    { id: "content", kind: "content", nodeId: "node", parentId: null, order: 0, locked: false },
  ];
  const tab = activateTestEditor(context, "video", "node");
  const other = activateTestEditor(context, "document", "other");
  const hooks = {
    ...context,
    ensureActiveSpacePersisted: async () => undefined,
    recordCommittedSpaceEdit: () => {},
    prepareContentRemoval: (roots: readonly string[]) => prepareEditorRemoval(context, roots),
  };
  return {
    context,
    tab,
    other,
    structure: createStructureActions(hooks),
    transfer: createStructureTransferActions(hooks),
  };
}

it.each(["delete", "move"])(
  "cancelling the affected editor prevents %s and retains all tabs",
  async (operation) => {
    const { context, tab, structure, transfer } = setup();
    onTestFinished(
      registerSpaceSessionParticipant({
        editorOwner: { spaceId: "inbox", tabId: tab.id, commitOnSwitch: false },
        commit: async () => {
          throw new ApplicationLeaveCancelled();
        },
      }),
    );
    const mutation =
      operation === "delete"
        ? structure.executeSpaceEdit({ kind: "structure.delete-subtrees", rootIds: ["content"] })
        : transfer.moveStructureNodes(["content"], "target");
    await expect(mutation).rejects.toBeInstanceOf(ApplicationLeaveCancelled);
    expect(mocks.execute).not.toHaveBeenCalled();
    expect(mocks.move).not.toHaveBeenCalled();
    expect(context.editorTabs.value).toHaveLength(2);
  },
);

it("does not discard or remove a prepared editor when deletion fails", async () => {
  const { context, tab, structure } = setup();
  const finalize = vi.fn<() => void>();
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "inbox", tabId: tab.id, commitOnSwitch: false },
      commit: async () => {},
      finalize,
    }),
  );
  const failure = new Error("disk full");
  mocks.execute.mockRejectedValueOnce(failure);
  await expect(
    structure.executeSpaceEdit({ kind: "structure.delete-subtrees", rootIds: ["content"] }),
  ).rejects.toBe(failure);
  expect(finalize).not.toHaveBeenCalled();
  expect(context.editorTabs.value).toHaveLength(2);
});

it("releases only affected tabs after a successful deletion", async () => {
  const { context, tab, other, structure } = setup();
  const finalize = vi.fn<() => void>();
  const unrelated = vi.fn<() => Promise<void>>();
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "inbox", tabId: tab.id, commitOnSwitch: false },
      commit: async () => {},
      finalize,
    }),
  );
  onTestFinished(
    registerSpaceSessionParticipant({
      editorOwner: { spaceId: "inbox", tabId: other.id, commitOnSwitch: true },
      commit: unrelated,
    }),
  );
  mocks.execute.mockResolvedValueOnce({
    mutationId: "mutation",
    appliedAtRevision: 1,
    selectionSuggestion: [],
  });
  mocks.projection.mockResolvedValueOnce({
    nodes: [],
    relations: [],
    scenes: [],
    editSnapshot: { editRevision: 1, structure: [], layouts: [], relationStyles: [] },
  });
  await structure.executeSpaceEdit({ kind: "structure.delete-subtrees", rootIds: ["content"] });
  expect(finalize).toHaveBeenCalledOnce();
  expect(unrelated).not.toHaveBeenCalled();
  expect(context.editorTabs.value).toEqual([other]);
});
