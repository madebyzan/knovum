import { describe, expect, it, vi } from "vitest";
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
vi.mock("../assets/asset-operations", () => ({
  cleanupDeletedSpaceAssets: assets.cleanup,
  removeSpaceAsset: assets.remove,
}));
vi.mock("../assets/canvas-thumbnail-file", () => ({ saveCanvasThumbnailBlob: assets.save }));
vi.mock("../workspace/space-operations", () => ({ updateSpaceThumbnail: repository.update }));
vi.mock("../canvas/editor-session", () => ({
  getCanvasEditorSession: () => ({
    activeEditor: ref({
      captureCanvasOverview: vi.fn<() => Promise<Blob>>().mockResolvedValue(new Blob(["canvas"])),
    }),
  }),
}));

import { commitActiveCanvasThumbnail } from "../canvas/thumbnail-scheduler";
import type { SpaceState } from "../session/session-state";

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
    } as unknown as SpaceState;

    await commitActiveCanvasThumbnail(state);

    expect(state.thumbnailDirty.value).toBe(true);
    expect(logs.warn).toHaveBeenCalledWith(expect.stringContaining("staged asset could not be removed"));
  });
});
