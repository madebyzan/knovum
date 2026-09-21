import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceScene } from "../../scenes/scene";
import {
  createSpace,
  materializeActiveSpace,
  openSpace,
  type SpaceSessionContext,
} from "../space-session-actions";
import { createTestSpace, createTestSessionContext } from "../../__tests__/test-session-context.test";

const nodeRepository = vi.hoisted(() => ({
  listNodes: vi.fn<(spaceId: string) => Promise<SpaceCanvasNode[]>>(),
}));
const lifecycleRepository = vi.hoisted(() => ({
  createSpace: vi.fn<() => Promise<void>>(),
  markSpaceOpened: vi.fn<(spaceId: string) => Promise<void>>(),
}));
const sceneDatabase = vi.hoisted(() => ({
  listScenes: vi.fn<(spaceId: string) => Promise<SpaceScene[]>>(),
}));
const relationRepository = vi.hoisted(() => ({
  listRelations: vi.fn<(spaceId: string) => Promise<SpaceRelation[]>>(),
}));

vi.mock("../../nodes/node-repository", () => ({
  listNodes: nodeRepository.listNodes,
}));
vi.mock("../space-repository", () => ({
  createSpace: lifecycleRepository.createSpace,
  listSpaces: vi.fn<() => void>(),
  markSpaceOpened: lifecycleRepository.markSpaceOpened,
  updateSpaceTitle: vi.fn<() => void>(),
}));
vi.mock("../../scenes/scene-repository", () => ({ listScenes: sceneDatabase.listScenes }));
vi.mock("../../knowledge/relations/relation-repository", () => ({
  listRelations: relationRepository.listRelations,
}));

function context(activeSpaceId = "inbox"): SpaceSessionContext {
  return createTestSessionContext({
    activeSpaceId: ref(activeSpaceId),
    spaces: ref([createTestSpace(), createTestSpace({ id: "research", kind: "space", title: "Research" })]),
  });
}

describe("openSpace", () => {
  beforeEach(() => {
    nodeRepository.listNodes.mockReset();
    sceneDatabase.listScenes.mockReset();
    relationRepository.listRelations.mockReset();
    relationRepository.listRelations.mockResolvedValue([]);
  });

  it("does not reload the already active space", async () => {
    const state = context();

    await openSpace(state, "inbox");
    await openSpace(state, "inbox");

    expect(nodeRepository.listNodes).not.toHaveBeenCalled();
    expect(sceneDatabase.listScenes).not.toHaveBeenCalled();
    expect(state.loading.value).toBe(false);
  });

  it("coalesces repeated clicks while the same space is loading", async () => {
    let release!: (nodes: SpaceCanvasNode[]) => void;
    nodeRepository.listNodes.mockReturnValue(new Promise((resolve) => (release = resolve)));
    sceneDatabase.listScenes.mockResolvedValue([]);
    relationRepository.listRelations.mockResolvedValue([]);
    const state = context("other");

    const first = openSpace(state, "inbox");
    const second = openSpace(state, "inbox");
    await vi.waitFor(() => expect(nodeRepository.listNodes).toHaveBeenCalledTimes(1));

    release([]);
    await Promise.all([first, second]);
    expect(state.activeSpaceId.value).toBe("inbox");
    expect(state.loading.value).toBe(false);
  });
  it("keeps the current canvas snapshot visible until the new Space is ready", async () => {
    let release!: (nodes: SpaceCanvasNode[]) => void;
    nodeRepository.listNodes.mockReturnValue(new Promise((resolve) => (release = resolve)));
    sceneDatabase.listScenes.mockResolvedValue([]);
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

    release([
      {
        id: "new",
        type: "note",
        initialPosition: { x: 10, y: 10 },
        data: { kind: "note", title: "New", tiptapContent: { type: "doc" }, plainText: "" },
      },
    ]);
    await pending;
    expect(state.nodes.value.map((node) => node.id)).toEqual(["new"]);
  });

  it("cancels a pending switch when the user returns to the active Space", async () => {
    let release!: (nodes: SpaceCanvasNode[]) => void;
    nodeRepository.listNodes.mockReturnValue(new Promise((resolve) => (release = resolve)));
    sceneDatabase.listScenes.mockResolvedValue([]);
    relationRepository.listRelations.mockResolvedValue([]);
    const state = context("inbox");

    const pending = openSpace(state, "research");
    expect(state.loading.value).toBe(true);
    await openSpace(state, "inbox");
    expect(state.loading.value).toBe(false);

    release([]);
    await pending;
    expect(state.activeSpaceId.value).toBe("inbox");
    expect(state.runtime.pendingSpaceId).toBeUndefined();
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
    expect(state.openSpaceIds.value).toEqual(["inbox", draftId]);
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
