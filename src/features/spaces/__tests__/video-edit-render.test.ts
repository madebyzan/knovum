import { Channel, type InvokeArgs } from "@tauri-apps/api/core";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { isSpaceNodeOpenable } from "../canvas/selection-actions";
import type { RenderVideoEditRequest } from "../contracts/generated/RenderVideoEditRequest";
import type { VideoEditProgress } from "../contracts/generated/VideoEditProgress";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import type { useSpaceStore } from "../session/space-store";
import { createVideoEditProject } from "../video-editing/project";
import { renderVideoEdit } from "../video-editing/render";
import { deferred } from "./support/canvas-fixtures";

const mocks = vi.hoisted(() => ({
  current: undefined as unknown,
  allocate: vi.fn<() => Promise<string>>(),
  invoke: vi.fn<(command: string, args?: InvokeArgs) => Promise<unknown>>(),
  logError: vi.fn<(message: string) => Promise<void>>(),
  toastError: vi.fn<(message: string) => void>(),
  toastSuccess:
    vi.fn<(message: string, options?: { action: { label: string; onClick: () => void } }) => void>(),
  // oxfmt-ignore
  tasks: { start: vi.fn<(title: string, statusMessage: string, cancel: () => Promise<void>) => string>(), progress: vi.fn<(taskId: string, progress: number, status?: string) => void>(), succeed: vi.fn<(taskId: string) => void>(), fail: vi.fn<(taskId: string, message: string) => void>(), dismiss: vi.fn<(taskId: string) => void>(), describeSubject: vi.fn<(taskId: string, subject: unknown) => void>() },
}));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => mocks.current }));
vi.mock("vue-sonner", () => ({ toast: { error: mocks.toastError, success: mocks.toastSuccess } }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));
vi.mock("../../../foundation/tasks/task-store", () => ({ useTaskStore: () => mocks.tasks }));
vi.mock("../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));

const source: SpaceCanvasNode<SpaceVideoPayload> = {
  id: "source-node",
  type: "video",
  initialPosition: { x: 0, y: 0 },
  // oxfmt-ignore
  data: { kind: "video", title: "Source video", fileName: "source.mp4", assetPath: "/assets/source.mp4", sourceRevision: "source-revision" },
};
const pending = { ...source, id: "output-node" };
// oxfmt-ignore
const project = createVideoEditProject({ nodeId: source.id, sourceRevision: "source-revision", durationMs: 2_000, sourceWidth: 1080, sourceHeight: 1920 });

function createStore() {
  return (mocks.current = {
    activeSpaceId: "space-id",
    createPendingContentForSpace: vi
      .fn<ReturnType<typeof useSpaceStore>["createPendingContentForSpace"]>()
      .mockResolvedValue([pending]),
    finalizePendingContent: vi.fn<ReturnType<typeof useSpaceStore>["finalizePendingContent"]>(),
    applyNodeContentForSpace: vi.fn<ReturnType<typeof useSpaceStore>["applyNodeContentForSpace"]>(),
    refreshActiveSpace: vi
      .fn<ReturnType<typeof useSpaceStore>["refreshActiveSpace"]>()
      .mockResolvedValue(undefined),
    openSpace: vi.fn<ReturnType<typeof useSpaceStore>["openSpace"]>().mockResolvedValue(),
    openSpaceCanvas: vi.fn<ReturnType<typeof useSpaceStore>["openSpaceCanvas"]>().mockResolvedValue(),
    requestReveal: vi.fn<ReturnType<typeof useSpaceStore>["requestReveal"]>(),
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  createStore();
  mocks.allocate.mockReset();
  mocks.allocate.mockResolvedValue("/assets/edited-reserved.mp4");
  mocks.invoke.mockReset();
  mocks.logError.mockResolvedValue();
  mocks.tasks.start.mockReturnValue("task-id");
  mockIPC((command, args) =>
    command === "plugin:spaces|allocate_video_edit_output_path"
      ? mocks.allocate()
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
    await renderVideoEdit("space-id", source, project);
    expect(store.createPendingContentForSpace).toHaveBeenCalledWith("space-id", {
      kind: "video",
      title: "Source video-edited.mp4",
      fileName: "source-edited.mp4",
      assetPath: "/assets/edited-reserved.mp4",
      processingState: "processing",
    });
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
      "spaces.media.videoEditor.render.stages.renderingVideoEdit",
    );
    expect(store.applyNodeContentForSpace).toHaveBeenCalledWith("space-id", pending.id, source.data);
    expect(store.finalizePendingContent).toHaveBeenCalledWith("space-id", [pending], { reveal: false });
    expect(mocks.tasks.succeed).toHaveBeenCalledWith("task-id");
    expect(store.openSpace).not.toHaveBeenCalled();
  });

  it.each([
    { nodeId: "wrong-node", data: source.data },
    { nodeId: pending.id, data: { kind: "markdown", title: "Not a video", text: "" } },
    { nodeId: pending.id, data: { ...source.data, assetPath: "" } },
  ])("does not publish invalid render output: %j", async (completed) => {
    const store = createStore();
    mocks.invoke.mockResolvedValue(completed);
    await renderVideoEdit("space-id", source, project);
    expect(store.applyNodeContentForSpace).not.toHaveBeenCalled();
    expect(store.openSpace).not.toHaveBeenCalled();
    expect(mocks.tasks.succeed).not.toHaveBeenCalled();
    expect(mocks.tasks.fail).toHaveBeenCalledWith("task-id", expect.any(String));
    expect(mocks.toastError).toHaveBeenCalled();
  });

  it("keeps cancellation in the video capability and dismisses the cancelled task", async () => {
    mocks.invoke.mockRejectedValueOnce({ code: "video_edit_render_cancelled", message: "Cancelled" });
    await renderVideoEdit("space-id", source, project);
    expect(mocks.tasks.dismiss).toHaveBeenCalledWith("task-id");
    expect(mocks.toastError).not.toHaveBeenCalled();
    mocks.invoke.mockResolvedValueOnce(undefined);
    const cancel = mocks.tasks.start.mock.calls[0]?.[2];
    await cancel?.();
    const request = (mocks.invoke.mock.calls[0]?.[1] as { request: RenderVideoEditRequest } | undefined)
      ?.request;
    expect(mocks.invoke).toHaveBeenLastCalledWith("plugin:spaces|cancel_video_edit_render", {
      derivationId: request?.id,
    });
  });

  it("keeps a committed render successful when refreshing the Space fails", async () => {
    const store = createStore();
    vi.mocked(store.refreshActiveSpace).mockRejectedValueOnce(new Error("Refresh failed"));
    mocks.invoke.mockResolvedValueOnce({ nodeId: pending.id, data: source.data });
    await expect(renderVideoEdit("space-id", source, project)).resolves.toBeUndefined();
    expect(mocks.tasks.succeed).toHaveBeenCalledWith("task-id");
    expect(mocks.tasks.fail).not.toHaveBeenCalled();
    expect(mocks.logError).toHaveBeenCalledWith(expect.stringContaining("Refresh failed"));
  });

  it.each(["other-video", "reopened"])("preserves a %s editor after a late render", async (target) => {
    const allocation = deferred<string>();
    mocks.allocate.mockReturnValueOnce(allocation.promise);
    mocks.invoke.mockResolvedValueOnce({ nodeId: pending.id, data: source.data });
    const store = createStore();
    const rendering = renderVideoEdit("space-id", source, project);
    await vi.waitFor(() => expect(mocks.allocate).toHaveBeenCalledOnce());

    store.activeSpaceId = target === "reopened" ? "space-id" : "space-b";
    const newerStore = createStore();
    allocation.resolve("/assets/edited-reserved.mp4");
    await rendering;

    expect(store.createPendingContentForSpace).toHaveBeenCalledWith(
      "space-id",
      expect.objectContaining({ assetPath: "/assets/edited-reserved.mp4" }),
    );
    expect(store.finalizePendingContent).toHaveBeenCalledWith("space-id", [pending], { reveal: false });
    expect(store.refreshActiveSpace).toHaveBeenCalledTimes(target === "reopened" ? 1 : 0);
    expect(newerStore.refreshActiveSpace).not.toHaveBeenCalled();
    expect(store.openSpace).not.toHaveBeenCalled();
    expect(store.requestReveal).not.toHaveBeenCalled();
  });
});
