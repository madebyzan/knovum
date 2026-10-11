import type { Channel } from "@tauri-apps/api/core";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import type { useTaskStore } from "../../../foundation/tasks/task-store";
import { generateImageLayers } from "../knowledge-derivations/image-layers";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { deferred } from "./support/canvas-fixtures";
import { imageSourceNode } from "./support/image-operation-fixtures";

const context = vi.hoisted(() => ({
  store: {
    activeSpaceId: "origin",
    addContents: vi.fn<(nodes: SpaceCanvasNode[]) => Promise<SpaceCanvasNode[]>>(),
    refreshActiveSpace: vi.fn<() => Promise<void>>(),
  },
  tasks: {
    start: vi.fn<ReturnType<typeof useTaskStore>["start"]>(),
    succeed: vi.fn<ReturnType<typeof useTaskStore>["succeed"]>(),
    fail: vi.fn<ReturnType<typeof useTaskStore>["fail"]>(),
    setStatusMessage: vi.fn<ReturnType<typeof useTaskStore>["setStatusMessage"]>(),
  },
}));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.store }));
vi.mock("../../../foundation/tasks/task-store", () => ({ useTaskStore: () => context.tasks }));
vi.mock("../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("../canvas/editor-session", () => ({
  getCanvasEditorSession: () => ({ selection: { value: { nodePlacements: [{ id: "image", x: 0, y: 0 }] } } }),
}));
vi.mock("../knowledge-derivations/canvas-yield", () => ({ yieldToCanvas: async () => undefined }));

beforeEach(() => {
  vi.clearAllMocks();
  context.store.activeSpaceId = "origin";
  context.store.addContents.mockImplementation(async (nodes) => nodes);
  context.store.refreshActiveSpace.mockResolvedValue(undefined);
  context.tasks.start.mockReturnValue("task");
});
afterEach(() => clearMocks());

it("does not insert image layers into a Space selected while allocating paths", async () => {
  const allocation = deferred<string>();
  const allocate = vi.fn<() => Promise<string>>(() => allocation.promise);
  mockIPC(() => allocate());
  const generating = generateImageLayers(imageSourceNode(), 2);
  await vi.waitFor(() => expect(allocate).toHaveBeenCalledTimes(2));
  context.store.activeSpaceId = "other";
  allocation.resolve("/managed/output.png");
  await expect(generating).rejects.toThrow("Image layer target Space changed");
  expect(context.store.addContents).not.toHaveBeenCalled();
  expect(context.tasks.start).not.toHaveBeenCalled();
});

it("keeps execution and completion in the origin Space after output creation", async () => {
  context.store.addContents.mockImplementation(async (nodes) => {
    context.store.activeSpaceId = "other";
    return nodes;
  });
  const generate = vi.fn<(args: unknown) => unknown>((args) => {
    const { request, onStage } = args as { request: { spaceId: string }; onStage: Channel<string> };
    expect(request.spaceId).toBe("origin");
    onStage.onmessage("generating");
    return [];
  });
  mockIPC((command, args) =>
    command === "plugin:spaces|allocate_visualization_output_path" ? "/managed/output.png" : generate(args),
  );
  await expect(generateImageLayers(imageSourceNode(), 2)).resolves.toEqual([]);
  expect(generate).toHaveBeenCalledOnce();
  expect(context.tasks.setStatusMessage).toHaveBeenCalledWith("task", expect.stringContaining("generating"));
  expect(context.tasks.succeed).toHaveBeenCalledWith("task");
  expect(context.store.refreshActiveSpace).not.toHaveBeenCalled();
});
