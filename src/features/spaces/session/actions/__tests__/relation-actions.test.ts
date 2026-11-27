import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref, shallowReactive } from "vue";

import {
  combineRecentSpaceHistory,
  recordSpaceHistory,
  undoSpaceHistory,
  type SpaceHistoryState,
} from "../../../history/stack";
import type { SpaceRelation } from "../../../model/relations/relation";
import { removeSpaceRelations, type RelationContext } from "../relation-actions";

const mocks = vi.hoisted(() => ({
  createRelation: vi.fn<() => Promise<void>>(),
  updateRelation: vi.fn<() => Promise<void>>(),
  revokeRelation: vi.fn<() => Promise<void>>(),
  revokeRelations: vi.fn<() => Promise<void>>(),
  restoreRelation: vi.fn<() => Promise<void>>(),
  restoreRelations: vi.fn<() => Promise<void>>(),
}));

vi.mock("../../../knowledge/relations/relation-operations", () => mocks);

function createHistory(): SpaceHistoryState {
  return shallowReactive({
    undoStack: [],
    redoStack: [],
    busy: false,
    pendingMutations: 0,
  });
}

function makeRelation(id: string, spaceId: string): SpaceRelation {
  return {
    id,
    spaceId,
    sourceId: "node-a",
    targetId: "node-b",
    kind: "supports",
    rationale: "",
    status: "active",
    actorKind: "human",
    createdAt: 1,
    updatedAt: 1,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("relation-actions mutation barrier and space guard", () => {
  it("blocks concurrent undo while relation revocation is in flight and guards space switch", async () => {
    let resolveRevoke!: () => void;
    mocks.revokeRelations.mockReturnValue(
      new Promise<void>((resolve) => {
        resolveRevoke = resolve;
      }),
    );
    const relation = makeRelation("rel-1", "space-1");
    const otherSpaceRelation = makeRelation("rel-1", "space-2");
    const undoProbe = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    const history = createHistory();
    recordSpaceHistory(history, { undo: undoProbe, redo: async () => undefined });

    const context: RelationContext = {
      activeSpaceId: ref("space-1"),
      relations: ref([relation]),
      history,
    };

    const pending = removeSpaceRelations(context, ["rel-1"]);
    expect(history.pendingMutations).toBe(1);
    await undoSpaceHistory(history);
    expect(undoProbe).not.toHaveBeenCalled();

    // Switch spaces while revocation is still pending: state in space-2 must remain untouched.
    context.activeSpaceId.value = "space-2";
    context.relations.value = [otherSpaceRelation];
    resolveRevoke();
    await pending;

    expect(history.pendingMutations).toBe(0);
    expect(context.relations.value).toEqual([otherSpaceRelation]);
  });

  it("compensates already-undone steps when a compound undo fails midway", async () => {
    const history = createHistory();
    const events: string[] = [];
    recordSpaceHistory(history, {
      undo: async () => {
        events.push("undo-1-fail");
        throw new Error("boom");
      },
      redo: async () => {
        events.push("redo-1");
      },
    });
    recordSpaceHistory(history, {
      undo: async () => {
        events.push("undo-2");
      },
      redo: async () => {
        events.push("redo-2");
      },
    });
    combineRecentSpaceHistory(history, 0);

    await expect(undoSpaceHistory(history)).rejects.toThrow("boom");
    expect(events).toEqual(["undo-2", "undo-1-fail", "redo-2"]);
    expect(history.undoStack).toHaveLength(1);
  });
});
