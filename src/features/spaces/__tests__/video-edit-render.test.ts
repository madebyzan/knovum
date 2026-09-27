import { Channel, type InvokeArgs } from "@tauri-apps/api/core";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { useTaskStore } from "../../../foundation/tasks/task-store";
import type { RenderVideoEditRequest } from "../contracts/generated/RenderVideoEditRequest";
import type { VideoEditProgress } from "../contracts/generated/VideoEditProgress";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { createVideoEditProject } from "../video-editing/project";
import { renderVideoEdit } from "../video-editing/render";

const native = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: InvokeArgs) => Promise<unknown>>(),
  toastError: vi.fn<(message: string) => void>(),
  tasks: {
    start: vi.fn<(title: string, statusMessage: string, cancel: () => Promise<void>) => string>(),
    progress: vi.fn<ReturnType<typeof useTaskStore>["progress"]>(),
    succeed: vi.fn<ReturnType<typeof useTaskStore>["succeed"]>(),
    fail: vi.fn<ReturnType<typeof useTaskStore>["fail"]>(),
    dismiss: vi.fn<ReturnType<typeof useTaskStore>["dismiss"]>(),
  },
}));
vi.mock("vue-sonner", () => ({ toast: { error: native.toastError } }));
vi.mock("../../../foundation/tasks/task-store", () => ({ useTaskStore: () => native.tasks }));
vi.mock("../../../foundation/i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));

const source: SpaceCanvasNode<SpaceVideoPayload> = {
  id: "source-node",
  type: "video",
  initialPosition: { x: 0, y: 0 },
  data: {
    kind: "video",
    title: "Source video",
    fileName: "source.mp4",
    assetPath: "/assets/source.mp4",
    sourceRevision: "source-revision",
  },
};
const pending = { ...source, id: "output-node" };
const project = createVideoEditProject("source-revision", 2_000, 1080, 1920);

function createStore(): Parameters<typeof renderVideoEdit>[0] {
  return {
    activeSpaceId: "space-id",
    addContent: vi.fn<Parameters<typeof renderVideoEdit>[0]["addContent"]>().mockResolvedValue([pending]),
    applyNodeContentForSpace: vi.fn<Parameters<typeof renderVideoEdit>[0]["applyNodeContentForSpace"]>(),
    refreshActiveSpace: vi
      .fn<Parameters<typeof renderVideoEdit>[0]["refreshActiveSpace"]>()
      .mockResolvedValue(undefined),
    closeVideoEditor: vi.fn<Parameters<typeof renderVideoEdit>[0]["closeVideoEditor"]>(),
    requestReveal: vi.fn<Parameters<typeof renderVideoEdit>[0]["requestReveal"]>(),
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  native.invoke.mockReset();
  native.tasks.start.mockReturnValue("task-id");
  mockIPC((command, args) => native.invoke(command, args));
});
afterEach(() => clearMocks());

describe("video render IPC boundary", () => {
  it("sends the source revision, output identity and Channel, then publishes validated content", async () => {
    const store = createStore();
    native.invoke.mockImplementationOnce(async (_command, args) => {
      const progress = (args as { onProgress: Channel<VideoEditProgress> }).onProgress;
      expect(progress).toBeInstanceOf(Channel);
      if (progress instanceof Channel)
        progress.onmessage({ stage: "renderingVideoEdit", progress: 45 } satisfies VideoEditProgress);
      return { nodeId: pending.id, data: source.data };
    });
    await renderVideoEdit(store, source, project);
    expect(native.invoke).toHaveBeenCalledExactlyOnceWith("plugin:spaces|render_video_edit", {
      request: {
        id: expect.any(String),
        spaceId: "space-id",
        sourceNodeId: source.id,
        outputNodeId: pending.id,
        sourceRevision: "source-revision",
        project,
      } satisfies RenderVideoEditRequest,
      onProgress: expect.any(Channel),
    });
    expect(native.tasks.progress).toHaveBeenCalledWith(
      "task-id",
      45,
      "spaces.media.videoEditor.renderingVideoEdit",
    );
    expect(store.applyNodeContentForSpace).toHaveBeenCalledWith("space-id", pending.id, source.data);
    expect(native.tasks.succeed).toHaveBeenCalledWith("task-id");
    expect(store.closeVideoEditor).toHaveBeenCalledExactlyOnceWith();
  });

  it.each([
    { nodeId: "wrong-node", data: source.data },
    { nodeId: pending.id, data: { kind: "markdown", title: "Not a video", text: "" } },
    { nodeId: pending.id, data: { ...source.data, assetPath: "" } },
  ])("does not publish invalid render output: %j", async (completed) => {
    const store = createStore();
    native.invoke.mockResolvedValue(completed);
    await renderVideoEdit(store, source, project);
    expect(store.applyNodeContentForSpace).not.toHaveBeenCalled();
    expect(store.closeVideoEditor).not.toHaveBeenCalled();
    expect(native.tasks.succeed).not.toHaveBeenCalled();
    expect(native.tasks.fail).toHaveBeenCalledWith("task-id", expect.any(String));
    expect(native.toastError).toHaveBeenCalled();
  });

  it("keeps cancellation in the video capability and dismisses the cancelled task", async () => {
    native.invoke.mockRejectedValueOnce({ code: "video_edit_render_cancelled", message: "Cancelled" });
    await renderVideoEdit(createStore(), source, project);
    expect(native.tasks.dismiss).toHaveBeenCalledWith("task-id");
    expect(native.toastError).not.toHaveBeenCalled();
    native.invoke.mockResolvedValueOnce(undefined);
    const cancel = native.tasks.start.mock.calls[0]?.[2];
    await cancel?.();
    const request = (native.invoke.mock.calls[0]?.[1] as { request: RenderVideoEditRequest } | undefined)
      ?.request;
    expect(request).toBeDefined();
    expect(native.invoke).toHaveBeenLastCalledWith("plugin:spaces|cancel_video_edit_render", {
      derivationId: request?.id,
    });
  });
});
