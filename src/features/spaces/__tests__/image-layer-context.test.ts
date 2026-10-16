import type { Channel } from "@tauri-apps/api/core";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import type { useTaskStore } from "../../../foundation/tasks/task-store";
import { splitImageLayers } from "../knowledge-derivations/image-layers";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { deferred } from "./support/canvas-fixtures";
import { imageSourceNode } from "./support/image-operation-fixtures";

const context = vi.hoisted(() => ({
  store: {
    activeSpaceId: "origin",
    addContents: vi.fn<(nodes: SpaceCanvasNode[]) => Promise<SpaceCanvasNode[]>>(),
    refreshActiveSpace: vi.fn<() => Promise<void>>(),
    saveNodeContentForSpace: vi.fn<() => Promise<void>>(),
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
vi.mock("vue-sonner", () => ({ toast: { error: vi.fn<(message: string) => void>() } }));

const PLAN = { planId: "split", layerCount: 2 };

beforeEach(() => {
  vi.clearAllMocks();
  context.store.activeSpaceId = "origin";
  context.store.addContents.mockImplementation(async (nodes) => nodes);
  context.store.refreshActiveSpace.mockResolvedValue(undefined);
  context.store.saveNodeContentForSpace.mockResolvedValue(undefined);
  context.tasks.start.mockReturnValue("task");
});
afterEach(() => clearMocks());

it("does not insert image layers into a Space selected while allocating paths", async () => {
  const allocation = deferred<string>();
  const allocate = vi.fn<() => Promise<string>>(() => allocation.promise);
  mockIPC((command) => (command === "plugin:spaces|plan_image_layers" ? PLAN : allocate()));
  const splitting = splitImageLayers(imageSourceNode());
  await vi.waitFor(() => expect(allocate).toHaveBeenCalledTimes(PLAN.layerCount));
  context.store.activeSpaceId = "other";
  allocation.resolve("/managed/output.png");
  await expect(splitting).resolves.toBeUndefined();
  expect(context.store.addContents).not.toHaveBeenCalled();
  expect(context.tasks.fail).toHaveBeenCalledWith("task", expect.stringContaining("Space changed"));
});

it("creates one output node per discovered layer and publishes the plan that produced them", async () => {
  context.store.addContents.mockImplementation(async (nodes) => {
    context.store.activeSpaceId = "other";
    return nodes;
  });
  const publish = vi.fn<(args: unknown) => unknown>((args) => {
    const { request } = args as {
      request: { spaceId: string; planId: string; generationId: string; outputNodeIds: string[] };
    };
    expect(request.spaceId).toBe("origin");
    expect(request.planId).toBe(request.generationId);
    expect(request.outputNodeIds).toHaveLength(PLAN.layerCount);
    return [];
  });
  const discover = vi.fn<(args: unknown) => unknown>((args) => {
    const { request, onStage } = args as { request: { spaceId: string }; onStage: Channel<string> };
    expect(request.spaceId).toBe("origin");
    onStage.onmessage("analyzing");
    return PLAN;
  });
  mockIPC((command, args) => {
    if (command === "plugin:spaces|plan_image_layers") return discover(args);
    if (command === "plugin:spaces|generate_image_layers") return publish(args);
    return "/managed/output.png";
  });
  await expect(splitImageLayers(imageSourceNode())).resolves.toEqual([]);
  expect(discover).toHaveBeenCalledOnce();
  expect(publish).toHaveBeenCalledOnce();
  expect(context.tasks.setStatusMessage).toHaveBeenCalledWith("task", expect.stringContaining("analyzing"));
  expect(context.tasks.succeed).toHaveBeenCalledWith("task");
  expect(context.store.refreshActiveSpace).not.toHaveBeenCalled();
});
