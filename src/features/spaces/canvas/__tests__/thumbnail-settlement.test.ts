import { beforeEach, describe, expect, it, vi } from "vitest";
import { computed, ref } from "vue";

const assets = vi.hoisted(() => ({
  cleanup: vi.fn<(paths: readonly string[]) => Promise<void>>(),
  remove: vi.fn<(path: string) => Promise<void>>(),
  save: vi.fn<(blob: Blob) => Promise<string>>(),
}));
const repository = vi.hoisted(() => ({
  update: vi.fn<(spaceId: string, path: string) => Promise<string[]>>(),
}));
const logs = vi.hoisted(() => ({
  error: vi.fn<(message: string) => void>(),
  warn: vi.fn<(message: string) => void>(),
}));
vi.mock("@tauri-apps/plugin-log", () => ({ error: logs.error, warn: logs.warn }));
vi.mock("../../assets/asset-operations", () => ({
  cleanupDeletedSpaceAssets: assets.cleanup,
  removeSpaceAsset: assets.remove,
}));
vi.mock("../../assets/canvas-thumbnail-file", () => ({ saveCanvasThumbnailBlob: assets.save }));
vi.mock("../../workspace/space-operations", () => ({ updateSpaceThumbnail: repository.update }));
vi.mock("../editor-session", () => ({
  currentCanvasEditor: () => ({
    captureCanvasOverview: vi.fn<() => Promise<Blob>>().mockResolvedValue(new Blob(["canvas"])),
  }),
}));

import type { SpaceState } from "../../session/session-state";
import { captureActiveCanvasThumbnail } from "../thumbnail-scheduler";

beforeEach(() => vi.resetAllMocks());

function capturedState() {
  const original = { id: "source", kind: "space", title: "Source", thumbnailPath: null, updatedAt: 1 };
  const destination = {
    id: "destination",
    kind: "space",
    title: "Destination",
    thumbnailPath: null,
    updatedAt: 2,
  };
  const active = ref(original);
  const state = {
    activeSpace: computed(() => active.value),
    spaces: ref([original, destination]),
    thumbnailDirty: ref(true),
    editSnapshot: ref({ editRevision: 0 }),
    nodes: ref([]),
  } as unknown as SpaceState;
  return { state, active, original, destination };
}

describe("canvas thumbnail settlement", () => {
  it("restores dirty state when update and staged cleanup both fail", async () => {
    const primary = new Error("database unavailable");
    assets.save.mockResolvedValue("/staged-thumbnail.png");
    repository.update.mockRejectedValue(primary);
    assets.remove.mockRejectedValue(new Error("thumbnail locked"));
    const space = { id: "space", kind: "space", title: "Space", thumbnailPath: null };
    const state = {
      activeSpace: computed(() => space),
      spaces: ref([space]),
      thumbnailDirty: ref(true),
      editSnapshot: ref({}),
      nodes: ref([]),
    } as unknown as SpaceState;

    captureActiveCanvasThumbnail(state);
    await vi.waitFor(() => expect(logs.error).toHaveBeenCalled());

    expect(state.thumbnailDirty.value).toBe(true);
    expect(logs.error).toHaveBeenCalledWith(expect.stringContaining("staged asset could not be removed"));
  });
  it("returns before image work completes and publishes to the captured Space after navigation", async () => {
    const { state, active, original, destination } = capturedState();
    let release!: (path: string) => void;
    assets.save.mockReturnValue(
      new Promise<string>((resolve) => {
        release = resolve;
      }),
    );
    repository.update.mockResolvedValue([]);
    assets.cleanup.mockResolvedValue();
    expect(captureActiveCanvasThumbnail(state)).toBeUndefined();
    await vi.waitFor(() => expect(assets.save).toHaveBeenCalled());
    active.value = destination;
    release("/source-thumbnail.png");
    await vi.waitFor(() => expect(repository.update).toHaveBeenCalledWith("source", "/source-thumbnail.png"));
    expect(original.thumbnailPath).toBe("/source-thumbnail.png");
    expect(destination.thumbnailPath).toBeNull();
  });

  it("removes a result whose captured layout revision changed and keeps the cover dirty", async () => {
    const { state } = capturedState();
    let release!: (path: string) => void;
    assets.save.mockReturnValue(
      new Promise<string>((resolve) => {
        release = resolve;
      }),
    );
    assets.remove.mockResolvedValue();
    captureActiveCanvasThumbnail(state);
    await vi.waitFor(() => expect(assets.save).toHaveBeenCalled());
    state.editSnapshot.value = { ...state.editSnapshot.value, editRevision: 1 };
    release("/stale.png");
    await vi.waitFor(() => expect(assets.remove).toHaveBeenCalledWith("/stale.png"));
    expect(repository.update).not.toHaveBeenCalled();
    expect(state.thumbnailDirty.value).toBe(true);
  });
});
