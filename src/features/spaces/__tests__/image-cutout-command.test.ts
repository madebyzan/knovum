import { type InvokeArgs } from "@tauri-apps/api/core";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import type { useTaskStore } from "../../../foundation/tasks/task-store";
import { createImageCutout } from "../knowledge-derivations/image-cutout";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";

const native = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: InvokeArgs) => Promise<unknown>>(),
  toastError: vi.fn<(message: string) => void>(),
  tasks: {
    start: vi.fn<ReturnType<typeof useTaskStore>["start"]>(),
    setStatusMessage: vi.fn<ReturnType<typeof useTaskStore>["setStatusMessage"]>(),
    succeed: vi.fn<ReturnType<typeof useTaskStore>["succeed"]>(),
    fail: vi.fn<ReturnType<typeof useTaskStore>["fail"]>(),
  },
}));
vi.mock("vue-sonner", () => ({ toast: { error: native.toastError } }));
vi.mock("../../../foundation/tasks/task-store", () => ({ useTaskStore: () => native.tasks }));
vi.mock("../../../foundation/i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("../canvas/editor-session", () => ({
  useCanvasEditorSession: () => ({
    selection: { value: { nodePlacements: [{ id: "source", x: 10, y: 20 }] } },
  }),
}));
vi.mock("../knowledge-derivations/canvas-yield", () => ({ yieldToCanvas: async () => undefined }));

const source: SpaceCanvasNode<SpaceImagePayload> = {
  id: "source",
  type: "image",
  initialPosition: { x: 10, y: 20 },
  data: {
    kind: "image",
    title: "Photo",
    fileName: "photo.png",
    assetPath: "/assets/photo.png",
    sourceRevision: "asset:1",
  },
};
const pending: SpaceCanvasNode<SpaceImagePayload> = {
  ...source,
  id: "output",
  data: {
    kind: "image",
    title: "photo-cutout.png",
    fileName: "photo-cutout.png",
    assetPath: "/assets/reserved.png",
    processingState: "processing",
  },
};

function createStore(): Parameters<typeof createImageCutout>[0] {
  return {
    activeSpaceId: "space",
    nodes: [source, pending],
    addContent: vi.fn<Parameters<typeof createImageCutout>[0]["addContent"]>().mockResolvedValue([pending]),
    applyNodeContentForSpace: vi.fn<Parameters<typeof createImageCutout>[0]["applyNodeContentForSpace"]>(),
    refreshActiveSpace: vi
      .fn<Parameters<typeof createImageCutout>[0]["refreshActiveSpace"]>()
      .mockResolvedValue(undefined),
    requestReveal: vi.fn<Parameters<typeof createImageCutout>[0]["requestReveal"]>(),
    saveNodeContentForSpace: vi
      .fn<Parameters<typeof createImageCutout>[0]["saveNodeContentForSpace"]>()
      .mockResolvedValue(undefined),
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  native.tasks.start.mockReturnValue("task");
  mockIPC((command, args) => native.invoke(command, args));
});
afterEach(() => clearMocks());

it("submits one Rust cutout command after creating the existing canvas output", async () => {
  const store = createStore();
  native.invoke.mockImplementation(async (command: string) => {
    if (command === "plugin:spaces|check_image_cutout_source") return true;
    if (command === "plugin:spaces|allocate_visualization_output_path") return pending.data.assetPath;
    if (command === "plugin:spaces|run_image_cutout_derivation")
      return { nodeId: pending.id, data: { ...pending.data, processingState: undefined } };
    throw new Error(`Unexpected command: ${command}`);
  });
  await createImageCutout(store, source);
  expect(store.addContent).toHaveBeenCalledWith(pending.data, { x: 314, y: 20 }, { reveal: true });
  expect(native.invoke).toHaveBeenCalledWith("plugin:spaces|run_image_cutout_derivation", {
    request: {
      id: expect.any(String),
      spaceId: "space",
      sourceNodeId: source.id,
      outputNodeId: pending.id,
      sourceRevision: "asset:1",
      sourceEdit: null,
    },
  });
  expect(native.invoke).toHaveBeenCalledTimes(3);
  expect(native.tasks.succeed).toHaveBeenCalledWith("task");
});

it("marks a pending output failed when the combined Rust command cannot run", async () => {
  const store = createStore();
  native.invoke.mockImplementation(async (command: string) => {
    if (command === "plugin:spaces|check_image_cutout_source") return true;
    if (command === "plugin:spaces|allocate_visualization_output_path") return pending.data.assetPath;
    throw new Error("Cutout failed");
  });
  await createImageCutout(store, source);
  expect(store.saveNodeContentForSpace).toHaveBeenCalledWith("space", {
    id: pending.id,
    expectedData: pending.data,
    data: { ...pending.data, processingState: "failed" },
  });
  expect(native.tasks.fail).toHaveBeenCalledWith("task", expect.stringContaining("Cutout failed"));
});

it("records the task failure even when refreshing the failed output fails", async () => {
  const store = createStore();
  vi.mocked(store.refreshActiveSpace).mockRejectedValueOnce(new Error("Refresh failed"));
  native.invoke.mockImplementation(async (command: string) => {
    if (command === "plugin:spaces|check_image_cutout_source") return true;
    if (command === "plugin:spaces|allocate_visualization_output_path") return pending.data.assetPath;
    throw new Error("Cutout failed");
  });
  await expect(createImageCutout(store, source)).rejects.toThrow("Refresh failed");
  expect(native.tasks.fail).toHaveBeenCalledWith("task", expect.stringContaining("Cutout failed"));
  expect(native.toastError).toHaveBeenCalled();
});

it("does not apply a pending-node fallback to another active Space", async () => {
  const store = createStore();
  vi.mocked(store.refreshActiveSpace).mockImplementationOnce(async () => {
    store.activeSpaceId = "another-space";
  });
  native.invoke.mockImplementation(async (command: string) => {
    if (command === "plugin:spaces|check_image_cutout_source") return true;
    if (command === "plugin:spaces|allocate_visualization_output_path") return pending.data.assetPath;
    throw new Error("Cutout failed");
  });
  await createImageCutout(store, source);
  expect(store.saveNodeContentForSpace).not.toHaveBeenCalled();
  expect(native.tasks.fail).toHaveBeenCalledWith("task", expect.stringContaining("Cutout failed"));
});
