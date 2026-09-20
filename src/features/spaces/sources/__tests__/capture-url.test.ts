import { beforeEach, describe, expect, it, vi } from "vitest";

import type { MediaImportProgress } from "../../media/import/contract";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceMediaPayload } from "../../model/nodes/capabilities";
import { captureHarness, importedVideo, pendingImport, POSITION, URL } from "../../__tests__/capture-url-fixture";

type Point = { x: number; y: number };
type WebsiteCapture = (
  context: object,
  url: string,
  position?: Point,
) => Promise<SpaceCanvasNode | undefined>;

const mocks = vi.hoisted(() => {
  class MediaImportCancelledError extends Error {}
  return {
    addSavedWebsite: vi.fn<WebsiteCapture>(),
    acquireMediaSource:
      vi.fn<
        (
          url: string,
          onProgress: (progress: MediaImportProgress) => void,
        ) => Promise<{ outcome: "imported"; payload: SpaceMediaPayload } | { outcome: "unsupported" }>
      >(),
    logWarn: vi.fn<(message: string) => void>(),
    logInfo: vi.fn<(message: string) => void>(),
    removeSpaceAssets: vi.fn<(paths: readonly string[]) => Promise<void>>(),
    toastError: vi.fn<(message: string) => void>(),
    MediaImportCancelledError,
  };
});

vi.mock("@tauri-apps/plugin-log", () => ({ info: mocks.logInfo, warn: mocks.logWarn }));
vi.mock("vue-sonner", () => ({ toast: { error: mocks.toastError } }));
vi.mock("../../../../foundation/i18n/i18n", () => ({
  i18n: { global: { t: (key: string) => key } },
}));
vi.mock("../../assets/asset-operations", () => ({ removeSpaceAssets: mocks.removeSpaceAssets }));
vi.mock("../../media/import/workflow", () => ({
  acquireMediaSource: mocks.acquireMediaSource,
  MediaImportCancelledError: mocks.MediaImportCancelledError,
}));
vi.mock("../../website/capture", () => ({ addSavedWebsite: mocks.addSavedWebsite }));

import { captureSpaceUrl } from "../capture-url";

describe("Space URL capture", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.addSavedWebsite.mockResolvedValue(undefined);
    mocks.removeSpaceAssets.mockResolvedValue(undefined);
  });

  it("shows a transient projection before resolution and persists one media node", async () => {
    const deferred = pendingImport();
    mocks.acquireMediaSource.mockReturnValue(deferred.promise);
    const harness = captureHarness();
    const capture = captureSpaceUrl(harness.context, URL, POSITION);

    expect(harness.begin).toHaveBeenCalledWith({
      spaceId: "space-1",
      url: URL,
      title: "youtube.com",
      position: POSITION,
      status: "resolving",
      downloadedBytes: 0,
      totalBytes: null,
      processedMs: null,
      durationMs: null,
    });
    expect(harness.addItem).not.toHaveBeenCalled();
    expect(mocks.logInfo).toHaveBeenCalledWith("[spaces.url-capture] route=media");

    deferred.resolve({ outcome: "imported", payload: importedVideo() });
    await capture;
    expect(harness.addItem).toHaveBeenCalledOnce();
    expect(harness.addItem).toHaveBeenCalledWith(importedVideo(), POSITION);
    expect(mocks.logInfo).toHaveBeenCalledWith("[spaces.url-capture] media node saved; created=true");
    expect(harness.finish).toHaveBeenCalledWith("import-1");
  });

  it("creates an ordinary website immediately without invoking media acquisition", async () => {
    const harness = captureHarness();
    const websiteUrl = "https://www.chinagoods.com/";
    await captureSpaceUrl(harness.context, websiteUrl, POSITION);
    expect(mocks.addSavedWebsite).toHaveBeenCalledWith(
      expect.objectContaining({ activeSpaceId: "space-1" }),
      websiteUrl,
      POSITION,
    );
    expect(mocks.acquireMediaSource).not.toHaveBeenCalled();
    expect(harness.begin).toHaveBeenCalledOnce();
    expect(harness.finish).toHaveBeenCalledWith("import-1");
  });

  it("sends the reported YouTube watch URL to media acquisition, not website capture", async () => {
    const harness = captureHarness();
    const watchUrl = "https://www.youtube.com/watch?v=CmjNjWeRE0Y";
    mocks.acquireMediaSource.mockResolvedValue({ outcome: "imported", payload: importedVideo() });
    await captureSpaceUrl(harness.context, watchUrl, POSITION);
    expect(mocks.acquireMediaSource).toHaveBeenCalledWith(watchUrl, expect.any(Function));
    expect(mocks.addSavedWebsite).not.toHaveBeenCalled();
  });

  it("reports an unsupported explicit media source without creating a website", async () => {
    mocks.acquireMediaSource.mockResolvedValue({ outcome: "unsupported" });
    const harness = captureHarness();
    await captureSpaceUrl(harness.context, URL, POSITION);
    expect(mocks.addSavedWebsite).not.toHaveBeenCalled();
    expect(mocks.toastError).toHaveBeenCalledWith("spaces.mediaImport.importFailed");
    expect(harness.update).toHaveBeenCalledWith("import-1", { status: "failed" });
    expect(harness.finish).not.toHaveBeenCalled();
  });

  it("keeps a failed capture visible instead of silently removing it", async () => {
    mocks.acquireMediaSource.mockRejectedValue(new Error("runtime unavailable"));
    const harness = captureHarness();
    await captureSpaceUrl(harness.context, URL, POSITION);
    expect(harness.update).toHaveBeenCalledWith("import-1", { status: "failed" });
    expect(harness.finish).not.toHaveBeenCalled();
    expect(harness.addItem).not.toHaveBeenCalled();
    expect(mocks.toastError).toHaveBeenCalledWith("spaces.mediaImport.importFailed");
  });

  it("updates canvas capture progress from the media task", async () => {
    const deferred = pendingImport();
    mocks.acquireMediaSource.mockReturnValue(deferred.promise);
    const harness = captureHarness();
    const capture = captureSpaceUrl(harness.context, URL, POSITION);
    const onProgress = mocks.acquireMediaSource.mock.calls[0]?.[1];
    onProgress?.({
      stage: "downloading",
      downloadedBytes: 512,
      totalBytes: 1024,
      processedMs: null,
      durationMs: null,
    });
    expect(harness.update).toHaveBeenCalledWith("import-1", {
      status: "downloading",
      downloadedBytes: 512,
      totalBytes: 1024,
      processedMs: null,
      durationMs: null,
    });
    deferred.resolve({ outcome: "imported", payload: importedVideo() });
    await capture;
  });

  it("removes the projection quietly when the import is cancelled", async () => {
    mocks.acquireMediaSource.mockRejectedValue(new mocks.MediaImportCancelledError());
    const harness = captureHarness();
    await captureSpaceUrl(harness.context, URL, POSITION);
    expect(harness.finish).toHaveBeenCalledWith("import-1");
    expect(mocks.toastError).not.toHaveBeenCalled();
  });

  it("does not persist into another space after navigation", async () => {
    const deferred = pendingImport();
    mocks.acquireMediaSource.mockReturnValue(deferred.promise);
    const harness = captureHarness();
    const capture = captureSpaceUrl(harness.context, URL, POSITION);
    await vi.waitFor(() => expect(mocks.acquireMediaSource).toHaveBeenCalledOnce());
    harness.setActiveSpace("space-2");
    deferred.resolve({ outcome: "imported", payload: importedVideo() });
    await capture;
    expect(harness.addItem).not.toHaveBeenCalled();
    expect(mocks.removeSpaceAssets).toHaveBeenCalledWith(["/spaces/assets/video.mp4"]);
    expect(harness.finish).toHaveBeenCalledWith("import-1");
  });

  it("removes downloaded assets when the media node cannot be persisted", async () => {
    mocks.acquireMediaSource.mockResolvedValue({ outcome: "imported", payload: importedVideo() });
    const harness = captureHarness();
    harness.addItem.mockRejectedValue(new Error("revision rejected"));
    await captureSpaceUrl(harness.context, URL, POSITION);
    expect(mocks.removeSpaceAssets).toHaveBeenCalledWith(["/spaces/assets/video.mp4"]);
    expect(harness.update).toHaveBeenCalledWith("import-1", { status: "failed" });
    expect(harness.finish).not.toHaveBeenCalled();
    expect(mocks.toastError).toHaveBeenCalledWith("spaces.mediaImport.importFailed");
  });
});
