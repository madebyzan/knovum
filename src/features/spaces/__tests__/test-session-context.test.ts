import { describe, expect, it } from "vitest";
import { ref } from "vue";

import type { Space } from "../model/space";
import type { SpaceSessionContext } from "../workspace/space-session-actions";

export function createTestSpace(overrides: Partial<Space> = {}): Space {
  return {
    id: "inbox",
    kind: "inbox",
    title: "Inbox",
    archivedAt: null,
    favorite: false,
    thumbnailPath: null,
    nodeCount: 0,
    lastOpenedAt: 1,
    createdAt: 1,
    updatedAt: 1,
    ...overrides,
  };
}

function defaultSessionContext(): SpaceSessionContext {
  return {
    spaces: ref([createTestSpace()]),
    draftSpace: ref(),
    activeSpaceId: ref("inbox"),
    nodes: ref([]),
    editSnapshot: ref({ editRevision: 0, structure: [], layouts: [], relationStyles: [] }),
    groupScopePath: ref([]),
    scenes: ref([]),
    relations: ref([]),
    ready: ref(true),
    loading: ref(false),
    focusedEditorId: ref(),
    history: { undoStack: [], redoStack: [], busy: false, pendingMutations: 0 },
    runtime: { openGeneration: 0 },
    openSpaceIds: ref(["inbox"]),
    closedSpaceIds: ref([]),
    navigationHistory: ref(["inbox"]),
    navigationIndex: ref(0),
  };
}

export function createTestSessionContext(overrides: Partial<SpaceSessionContext> = {}): SpaceSessionContext {
  return Object.assign(defaultSessionContext(), overrides);
}

describe("test-session-context", () => {
  it("creates a valid default space and session context", () => {
    const space = createTestSpace();
    expect(space.id).toBe("inbox");
    const context = createTestSessionContext();
    expect(context.activeSpaceId.value).toBe("inbox");
    expect(context.spaces.value).toHaveLength(1);
  });
});
