import { flushPromises } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { createTestSpace, createTestSessionContext } from "../../__tests__/support/session-context";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { loadSpaceSessionProjection } from "../../session/space-projection";
import { materializeActiveSpace } from "../draft-space-actions";
import type { SpaceSessionContext } from "../space-session-context";
import { createSpace, openSpace } from "../workbench-navigation-actions";

type Projection = Awaited<ReturnType<typeof loadSpaceSessionProjection>>;
const projectionRepository = vi.hoisted(() => ({
  load: vi.fn<(spaceId: string) => Promise<Projection>>(),
}));
const lifecycleRepository = vi.hoisted(() => ({
  createSpace: vi.fn<() => Promise<void>>(),
  markSpaceOpened: vi.fn<(spaceId: string) => Promise<void>>(),
}));
vi.mock("../../session/space-projection", () => ({
  loadSpaceSessionProjection: projectionRepository.load,
}));
vi.mock("../space-operations", () => ({
  createSpace: lifecycleRepository.createSpace,
  listSpaces: vi.fn<() => void>(),
  markSpaceOpened: lifecycleRepository.markSpaceOpened,
  updateSpaceTitle: vi.fn<() => void>(),
}));
function projection(nodes: SpaceCanvasNode[] = []): Projection {
  return {
    nodes,
    scenes: [],
    relations: [],
    editSnapshot: { editRevision: 0, structure: [], layouts: [], relationStyles: [] },
  };
}

function context(activeSpaceId = "inbox"): SpaceSessionContext {
  return createTestSessionContext({
    activeSpaceId,
    spaces: [createTestSpace(), createTestSpace({ id: "research", kind: "space", title: "Research" })],
  });
}

describe("openSpace", () => {
  beforeEach(() => {
    projectionRepository.load.mockReset();
    lifecycleRepository.markSpaceOpened.mockReset().mockResolvedValue();
  });

  it("does not reload the already active space", async () => {
    const state = context();

    await openSpace(state, "inbox");
    await openSpace(state, "inbox");

    expect(projectionRepository.load).not.toHaveBeenCalled();
    expect(state.loading.value).toBe(false);
  });

  it("coalesces repeated clicks while the same space is loading", async () => {
    let release!: (value: Projection) => void;
    projectionRepository.load.mockReturnValue(new Promise((resolve) => (release = resolve)));

    const state = context("other");

    const first = openSpace(state, "inbox");
    const second = openSpace(state, "inbox");
    await flushPromises();
    expect(projectionRepository.load).toHaveBeenCalledTimes(1);

    release(projection());
    await Promise.all([first, second]);
    expect(state.activeSpaceId.value).toBe("inbox");
    expect(state.loading.value).toBe(false);
  });
  it("keeps the current canvas snapshot visible until the new Space is ready", async () => {
    let release!: (value: Projection) => void;
    projectionRepository.load.mockReturnValue(new Promise((resolve) => (release = resolve)));

    const state = context("inbox");
    state.nodes.value = [
      {
        id: "old",
        type: "note",
        initialPosition: { x: 0, y: 0 },
        data: { kind: "note", title: "Old", tiptapContent: { type: "doc" }, plainText: "" },
      },
    ];

    const pending = openSpace(state, "research");
    expect(state.nodes.value.map((node) => node.id)).toEqual(["old"]);
    await flushPromises();

    release(
      projection([
        {
          id: "new",
          type: "note",
          initialPosition: { x: 10, y: 10 },
          data: { kind: "note", title: "New", tiptapContent: { type: "doc" }, plainText: "" },
        },
      ]),
    );
    await pending;
    expect(state.nodes.value.map((node) => node.id)).toEqual(["new"]);
  });

  it("cancels a pending switch when the user returns to the active Space", async () => {
    let release!: (value: Projection) => void;
    projectionRepository.load.mockReturnValue(new Promise((resolve) => (release = resolve)));

    const state = context("inbox");

    const pending = openSpace(state, "research");
    expect(state.loading.value).toBe(true);
    await flushPromises();
    await openSpace(state, "inbox");
    expect(state.loading.value).toBe(false);

    release(projection());
    await pending;
    expect(state.activeSpaceId.value).toBe("inbox");
    expect(state.runtime.pendingSpaceId).toBeUndefined();
  });

  it("leaves the active projection intact if recording the new Space fails", async () => {
    projectionRepository.load.mockResolvedValue(projection());
    lifecycleRepository.markSpaceOpened.mockRejectedValueOnce(new Error("database unavailable"));
    const state = context("inbox");
    const research = state.spaces.value.find((space) => space.id === "research")!;
    const lastOpenedAt = research.lastOpenedAt;

    await expect(openSpace(state, "research")).rejects.toThrow("database unavailable");

    expect(state.activeSpaceId.value).toBe("inbox");
    expect(state.tabs.value.map(({ spaceId }) => spaceId)).toEqual(["inbox"]);
    expect(research.lastOpenedAt).toBe(lastOpenedAt);
    expect(state.loading.value).toBe(false);
  });
});

describe("draft Space creation", () => {
  beforeEach(() => lifecycleRepository.createSpace.mockReset().mockResolvedValue());

  it("reuses one session draft without persisting repeated new-Space actions", async () => {
    const state = context();

    await createSpace(state, "Untitled Space");
    const draftId = state.draftSpace.value?.id;
    await createSpace(state, "Untitled Space");

    expect(draftId).toBeTruthy();
    expect(state.activeSpaceId.value).toBe(draftId);
    expect(state.tabs.value.map(({ spaceId }) => spaceId)).toEqual(["inbox", draftId]);
    expect(state.spaces.value).toHaveLength(2);
    expect(lifecycleRepository.createSpace).not.toHaveBeenCalled();
  });

  it("persists the draft exactly once before its first content write", async () => {
    const state = context();
    await createSpace(state, "Untitled Space");
    const draft = state.draftSpace.value;

    await materializeActiveSpace(state);
    await materializeActiveSpace(state);

    expect(lifecycleRepository.createSpace).toHaveBeenCalledTimes(1);
    expect(lifecycleRepository.createSpace).toHaveBeenCalledWith(draft);
    expect(state.draftSpace.value).toBeUndefined();
    expect(state.spaces.value).toContain(draft);
  });
});
