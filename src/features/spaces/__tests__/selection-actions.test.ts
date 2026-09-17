import { describe, expect, it, vi } from "vitest";
import { shallowRef, type Ref } from "vue";

import { createSpaceNode } from "../canvas/node";
import {
  combineRecentSpaceHistory,
  recordSpaceHistory,
  undoSpaceHistory,
  type SpaceHistoryState,
} from "../history/stack";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { createSelectionActions } from "../organization/selection-actions";
import type { NodeDataChange } from "../session/actions/node-mutations";

function harness(locked = false) {
  const section = createSpaceNode(
    { kind: "section", title: "Research", ...(locked ? { locked: true } : {}) },
    { x: 0, y: 0 },
    "section",
  );
  const member = createSpaceNode(
    { kind: "note", title: "Idea", tiptapContent: { type: "doc" }, plainText: "", sectionId: "section" },
    { x: 80, y: 90 },
    "member",
  );
  const nodes: Ref<SpaceCanvasNode[]> = shallowRef([section, member]);
  const changes: NodeDataChange[][] = [];
  const organization = createSelectionActions({
    nodes,
    addItem:
      vi.fn<
        (
          data: SpaceNodePayload,
          position?: { x: number; y: number },
          options?: { reveal?: boolean },
        ) => Promise<SpaceCanvasNode[]>
      >(),
    deleteItems: vi.fn<(ids: readonly string[]) => Promise<void>>(),
    removeRelations: vi.fn<(ids: readonly string[]) => Promise<void>>(),
    updateManyData: vi.fn<(items: readonly NodeDataChange[]) => Promise<void>>(async (items) => {
      changes.push([...items]);
    }),
    recordHistoryMutation: vi.fn<(action: () => Promise<void>) => Promise<void>>(async (action) => action()),
  });
  return { organization, changes };
}

describe("spaces organization", () => {
  it("allows a Section itself to be locked", async () => {
    const { organization, changes } = harness();
    await organization.setLocked(["section"], true);

    expect(changes[0]).toHaveLength(1);
    expect(changes[0]?.[0]?.id).toBe("section");
    expect(changes[0]?.[0]?.after.locked).toBe(true);
  });
});

function historyHarness() {
  const history: SpaceHistoryState = {
    undoStack: [],
    redoStack: [],
    busy: false,
    pendingMutations: 0,
  };
  const order: string[] = [];
  const undone: string[] = [];
  const nodes: Ref<SpaceCanvasNode[]> = shallowRef([
    createSpaceNode(
      { kind: "note", title: "Idea", tiptapContent: { type: "doc" }, plainText: "" },
      { x: 0, y: 0 },
      "member",
    ),
  ]);
  const record = (label: string) =>
    recordSpaceHistory(history, {
      undo: async () => {
        undone.push(label);
      },
      redo: async () => {},
    });
  const organization = createSelectionActions({
    nodes,
    addItem: async () => [],
    updateManyData: async () => {},
    deleteItems: async (ids) => {
      if (!ids.length) return;
      order.push(`items:${ids.join(",")}`);
      record("items");
    },
    removeRelations: async (ids) => {
      if (!ids.length) return;
      order.push(`relations:${ids.join(",")}`);
      record("relations");
    },
    recordHistoryMutation: async (action) => {
      const startIndex = history.undoStack.length;
      try {
        await action();
      } finally {
        combineRecentSpaceHistory(history, startIndex);
      }
    },
  });
  return { organization, history, order, undone };
}

describe("deleteSelection", () => {
  it("removes selected relations before nodes and undoes both in one step", async () => {
    const { organization, history, order, undone } = historyHarness();
    await organization.deleteSelection(["member"], ["relation"]);

    expect(order).toEqual(["relations:relation", "items:member"]);
    expect(history.undoStack).toHaveLength(1);
    await undoSpaceHistory(history);
    expect(undone).toEqual(["items", "relations"]);
    expect(history.redoStack).toHaveLength(1);
  });

  it("deletes a selected relation on its own", async () => {
    const { organization, history, order, undone } = historyHarness();
    await organization.deleteSelection([], ["relation"]);

    expect(order).toEqual(["relations:relation"]);
    expect(history.undoStack).toHaveLength(1);
    await undoSpaceHistory(history);
    expect(undone).toEqual(["relations"]);
  });
});
