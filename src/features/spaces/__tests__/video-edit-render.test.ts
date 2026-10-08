import { Channel, type InvokeArgs } from "@tauri-apps/api/core";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { useTaskStore } from "../../../foundation/tasks/task-store";
import { isSpaceNodeOpenable } from "../canvas/selection-actions";
import type { RenderVideoEditRequest } from "../contracts/generated/RenderVideoEditRequest";
import type { VideoEditProgress } from "../contracts/generated/VideoEditProgress";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { createVideoEditProject } from "../video-editing/project";
import { renderVideoEdit } from "../video-editing/render";

const mocks = vi.hoisted(() => ({
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
vi.mock("vue-sonner", () => ({ toast: { error: mocks.toastError } }));
vi.mock("../../../foundation/tasks/task-store", () => ({ useTaskStore: () => mocks.tasks }));
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
const project = createVideoEditProject({
  sourceRevision: "source-revision",
  durationMs: 2_000,
  sourceWidth: 1080,
  sourceHeight: 1920,
});

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
  mocks.invoke.mockReset();
  mocks.tasks.start.mockReturnValue("task-id");
  mockIPC((command, args) =>
    command === "plugin:spaces|allocate_video_edit_output_path"
      ? "/assets/edited-reserved.mp4"
      : mocks.invoke(command, args),
  );
});
afterEach(() => clearMocks());

describe("video render IPC boundary", () => {
  it("does not expose an unpublished video output as openable", () => {
    expect(isSpaceNodeOpenable(source)).toBe(true);
    expect(isSpaceNodeOpenable({ ...source, data: { ...source.data, sourceRevision: undefined } })).toBe(
      true,
    );
    expect(
      isSpaceNodeOpenable({
        ...source,
        id: "pending-output",
        data: { ...source.data, sourceRevision: undefined, processingState: "processing" },
      }),
    ).toBe(false);
  });

  it("sends the source revision, output identity and Channel, then publishes validated content", async () => {
    const store = createStore();
    mocks.invoke.mockImplementationOnce(async (_command, args) => {
      const progress = (args as { onProgress: Channel<VideoEditProgress> }).onProgress;
      expect(progress).toBeInstanceOf(Channel);
      if (progress instanceof Channel)
        progress.onmessage({ stage: "renderingVideoEdit", progress: 45 } satisfies VideoEditProgress);
      return { nodeId: pending.id, data: source.data };
    });
    await renderVideoEdit(store, source, project);
    expect(store.addContent).toHaveBeenCalledWith(
      expect.objectContaining({
        assetPath: "/assets/edited-reserved.mp4",
        processingState: "processing",
      }),
      undefined,
      { reveal: false },
    );
    const pendingContent = vi.mocked(store.addContent).mock.calls[0]?.[0];
    expect(pendingContent?.sourceRevision).toBeUndefined();
    expect(mocks.invoke).toHaveBeenCalledExactlyOnceWith("plugin:spaces|render_video_edit", {
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
    expect(mocks.tasks.progress).toHaveBeenCalledWith(
      "task-id",
      45,
      "spaces.media.videoEditor.renderingVideoEdit",
    );
    expect(store.applyNodeContentForSpace).toHaveBeenCalledWith("space-id", pending.id, source.data);
    expect(mocks.tasks.succeed).toHaveBeenCalledWith("task-id");
    expect(store.closeVideoEditor).toHaveBeenCalledExactlyOnceWith();
  });

  it.each([
    { nodeId: "wrong-node", data: source.data },
    { nodeId: pending.id, data: { kind: "markdown", title: "Not a video", text: "" } },
    { nodeId: pending.id, data: { ...source.data, assetPath: "" } },
  ])("does not publish invalid render output: %j", async (completed) => {
    const store = createStore();
    mocks.invoke.mockResolvedValue(completed);
    await renderVideoEdit(store, source, project);
    expect(store.applyNodeContentForSpace).not.toHaveBeenCalled();
    expect(store.closeVideoEditor).not.toHaveBeenCalled();
    expect(mocks.tasks.succeed).not.toHaveBeenCalled();
    expect(mocks.tasks.fail).toHaveBeenCalledWith("task-id", expect.any(String));
    expect(mocks.toastError).toHaveBeenCalled();
  });

  it("keeps cancellation in the video capability and dismisses the cancelled task", async () => {
    mocks.invoke.mockRejectedValueOnce({ code: "video_edit_render_cancelled", message: "Cancelled" });
    await renderVideoEdit(createStore(), source, project);
    expect(mocks.tasks.dismiss).toHaveBeenCalledWith("task-id");
    expect(mocks.toastError).not.toHaveBeenCalled();
    mocks.invoke.mockResolvedValueOnce(undefined);
    const cancel = mocks.tasks.start.mock.calls[0]?.[2];
    await cancel?.();
    const request = (mocks.invoke.mock.calls[0]?.[1] as { request: RenderVideoEditRequest } | undefined)
      ?.request;
    expect(request).toBeDefined();
    expect(mocks.invoke).toHaveBeenLastCalledWith("plugin:spaces|cancel_video_edit_render", {
      derivationId: request?.id,
    });
  });

  it("settles the failed task when refreshing the Space fails", async () => {
    const store = createStore();
    vi.mocked(store.refreshActiveSpace).mockRejectedValueOnce(new Error("Refresh failed"));
    mocks.invoke.mockRejectedValueOnce(new Error("Render failed"));
    await expect(renderVideoEdit(store, source, project)).rejects.toThrow("Refresh failed");
    expect(mocks.tasks.fail).toHaveBeenCalledWith("task-id", expect.stringContaining("Render failed"));
    expect(mocks.toastError).toHaveBeenCalled();
  });
});
