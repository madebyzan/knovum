import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref } from "vue";

import type { SpaceScene } from "../scenes/scene";

const sceneRepository = vi.hoisted(() => ({
  createScene: vi.fn<() => Promise<void>>(),
  deleteScene: vi.fn<(sceneId: string) => Promise<string[]>>(),
  updateScene: vi.fn<(scene: SpaceScene) => Promise<string[]>>(),
  updateSceneOrders: vi.fn<() => Promise<void>>(),
}));
const assets = vi.hoisted(() => ({
  cleanupDeletedSpaceAssets: vi.fn<(paths: readonly string[]) => Promise<void>>(),
}));

vi.mock("../scenes/scene-operations", () => sceneRepository);
vi.mock("../assets/asset-operations", () => assets);

import { removeSpaceScene, updateSpaceScene } from "../scenes/scene-actions";

function scene(): SpaceScene {
  return {
    id: "scene",
    spaceId: "space",
    title: "Before",
    thumbnailPath: "/old.png",
    viewportX: 0,
    viewportY: 0,
    viewportZoom: 1,
    orderIndex: 100,
    focusStructureNodeIds: [],
    relationKinds: [],
    relationStatuses: [],
    queryFilter: {},
    createdAt: 1,
    updatedAt: 1,
  };
}

function context(current: SpaceScene): Parameters<typeof updateSpaceScene>[0] {
  return {
    activeSpaceId: ref("space"),
    scenes: ref([current]),
    editSnapshot: ref({ editRevision: 0, structure: [], layouts: [], relationStyles: [] }),
    focusNodeIds: ref<string[]>([]),
    relationKinds: ref<SpaceScene["relationKinds"]>([]),
    relationStatuses: ref<SpaceScene["relationStatuses"]>([]),
    queryFilter: ref<SpaceScene["queryFilter"]>({}),
  };
}

describe("Saved View committed projections", () => {
  beforeEach(() => vi.clearAllMocks());

  it("projects an update before obsolete thumbnail cleanup settles", async () => {
    const current = scene();
    const state = context(current);
    sceneRepository.updateScene.mockResolvedValue(["/old.png"]);
    let finishCleanup!: () => void;
    assets.cleanupDeletedSpaceAssets.mockReturnValue(
      new Promise<void>((resolve) => (finishCleanup = resolve)),
    );

    const pending = updateSpaceScene(state, current.id, { title: "After" });
    await vi.waitFor(() => expect(state.scenes.value[0]?.title).toBe("After"));
    expect(assets.cleanupDeletedSpaceAssets).toHaveBeenCalledWith(["/old.png"]);

    finishCleanup();
    await expect(pending).resolves.toMatchObject({ title: "After" });
  });

  it("projects a deletion before obsolete thumbnail cleanup settles", async () => {
    const current = scene();
    const state = context(current);
    sceneRepository.deleteScene.mockResolvedValue(["/old.png"]);
    let finishCleanup!: () => void;
    assets.cleanupDeletedSpaceAssets.mockReturnValue(
      new Promise<void>((resolve) => (finishCleanup = resolve)),
    );

    const pending = removeSpaceScene(state, current.id);
    await vi.waitFor(() => expect(state.scenes.value).toEqual([]));
    expect(assets.cleanupDeletedSpaceAssets).toHaveBeenCalledWith(["/old.png"]);

    finishCleanup();
    await expect(pending).resolves.toBeUndefined();
  });
});
