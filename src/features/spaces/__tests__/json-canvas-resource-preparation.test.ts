import { beforeEach, describe, expect, it, vi } from "vitest";

import { prepareJsonCanvasNodes } from "../session/actions/json-canvas-node-preparation";

const assets = vi.hoisted(() => ({
  cancel: vi.fn<(jobId: string) => Promise<void>>(async () => {}),
  commit: vi.fn<(jobId: string, progress: (value: never) => void) => Promise<unknown>>(),
  prepare: vi.fn<(sourcePath: string) => Promise<unknown>>(),
  remove: vi.fn<(paths: readonly string[]) => Promise<void>>(async () => {}),
}));
const tasks = vi.hoisted(() => ({
  start: vi.fn<(title: string) => string>(() => crypto.randomUUID()),
  title: vi.fn<(id: string, title: string) => void>(),
  setCancellation: vi.fn<(id: string, action?: () => Promise<void>) => void>(),
  detail: vi.fn<(id: string, detail: string) => void>(),
  progress: vi.fn<(id: string, percent: number, detail: string) => void>(),
}));

vi.mock("@tauri-apps/api/path", () => ({
  isAbsolute: async (value: string) => value.startsWith("/"),
  resolve: async (...parts: string[]) => parts.join("/").replaceAll("//", "/"),
  sep: () => "/",
}));
vi.mock("../assets/asset-operations", () => ({
  cancelSpaceAssetImport: assets.cancel,
  copyStagedSpaceAsset: assets.commit,
  prepareSpaceAssetImport: assets.prepare,
  removeSpaceAssets: assets.remove,
}));
vi.mock("../../../foundation/tasks/task-store", () => ({ useTaskStore: () => tasks }));

describe("JSON Canvas resource preparation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    assets.prepare.mockImplementation(async (sourcePath) => ({
      jobId: sourcePath,
      asset: { kind: "file", name: sourcePath.split("/").at(-1) ?? "resource" },
    }));
    assets.commit.mockImplementation(async (sourcePath) => ({
      kind: sourcePath.endsWith("bg.png") ? "image" : "file",
      name: sourcePath.split("/").at(-1) ?? "resource",
      path: sourcePath.endsWith("bg.png") ? "/managed/bg.png" : "/managed/doc.pdf",
    }));
  });

  it("copies group backgrounds and file nodes through the same managed resource pipeline", async () => {
    const preparation = await prepareJsonCanvasNodes({
      sourceDirectory: "/source",
      baseRevision: 0,
      document: { version: 1, viewport: { x: 0, y: 0, zoom: 1 }, nodes: [] },
      plan: {
        nodes: [
          {
            sourceId: "group",
            frame: { x: 0, y: 0, width: 400, height: 300, rotation: 0 },
            zIndex: 0,
            appearance: { background: "assets/bg.png", backgroundStyle: "ratio" },
            content: { type: "group", label: "Group" },
          },
          {
            sourceId: "file",
            frame: { x: 20, y: 20, width: 200, height: 120, rotation: 0 },
            zIndex: 1,
            content: { type: "file", file: "assets/doc.pdf" },
          },
        ],
        edges: [],
      },
    });

    expect(assets.prepare.mock.calls.map(([path]) => path)).toEqual([
      "/source/assets/bg.png",
      "/source/assets/doc.pdf",
    ]);
    expect(preparation.backgroundPaths.get("group")).toBe("/managed/bg.png");
    expect(preparation.nodes[1]?.node.data).toMatchObject({ assetPath: "/managed/doc.pdf" });
    expect(preparation.assetPaths).toEqual(["/managed/bg.png", "/managed/doc.pdf"]);
  });
});
