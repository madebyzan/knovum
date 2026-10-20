import { beforeEach, describe, expect, it, vi } from "vitest";

import { createAppError } from "../../../foundation/errors/app-error";
import type { MediaAcquisitionSubject } from "../media/import/acquisition";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceMediaPayload } from "../model/nodes/capabilities";
import {
  ARRIVING_TITLE,
  captureHarness,
  importedVideo,
  pendingImport,
  POSITION,
  RESERVED_ASSET,
  URL,
} from "./support/capture-url-harness";

type Point = { x: number; y: number };
type Acquisition = (
  url: string,
  subject: MediaAcquisitionSubject,
) => Promise<{ outcome: "imported"; payload: SpaceMediaPayload } | { outcome: "unsupported" }>;

const mocks = vi.hoisted(() => {
  return {
    addSavedWebsite:
      vi.fn<(context: object, url: string, position?: Point) => Promise<SpaceCanvasNode | undefined>>(),
    acquireMediaSource: vi.fn<Acquisition>(),
    allocateMediaImportAsset: vi.fn<(jobId: string, kind: "audio" | "video") => Promise<string>>(),
    logWarn: vi.fn<(message: string) => void>(),
    logInfo: vi.fn<(message: string) => void>(),
    removeSpaceAssets: vi.fn<(paths: readonly string[]) => Promise<void>>(),
  };
});

vi.mock("@tauri-apps/plugin-log", () => ({ info: mocks.logInfo, warn: mocks.logWarn }));
vi.mock("../../../foundation/i18n/instance", () => ({
  i18n: { global: { t: (key: string) => key } },
}));
vi.mock("../assets/asset-operations", () => ({ removeSpaceAssets: mocks.removeSpaceAssets }));
vi.mock("../media/import/acquisition", () => ({
  acquireMediaSource: mocks.acquireMediaSource,
}));
vi.mock("../media/import/ipc", () => ({ allocateMediaImportAsset: mocks.allocateMediaImportAsset }));
vi.mock("../website/capture", () => ({ addSavedWebsite: mocks.addSavedWebsite }));

import { captureSpaceUrl } from "../sources/capture-url";

const ARRIVING_VIDEO = {
  kind: "video",
  title: ARRIVING_TITLE,
  assetPath: RESERVED_ASSET,
  fileName: "import-1.mp4",
  processingState: "importing",
} as const;

describe("Space URL capture", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.addSavedWebsite.mockResolvedValue(undefined);
    mocks.removeSpaceAssets.mockResolvedValue(undefined);
    mocks.allocateMediaImportAsset.mockResolvedValue(RESERVED_ASSET);
  });

  it("reserves the media node before the download starts, not when it ends", async () => {
    const deferred = pendingImport();
    mocks.acquireMediaSource.mockReturnValue(deferred.promise);
    const harness = captureHarness();
    const capture = captureSpaceUrl(harness.context, URL, POSITION);

    await vi.waitFor(() => expect(harness.createPending).toHaveBeenCalledOnce());
    expect(mocks.allocateMediaImportAsset).toHaveBeenCalledWith(expect.any(String), "video");
    expect(harness.createPending).toHaveBeenCalledWith("space-1", ARRIVING_VIDEO, POSITION);
    expect(mocks.acquireMediaSource).toHaveBeenCalledWith(URL, {
      jobId: expect.any(String),
      nodeId: "media",
      kind: "video",
      title: ARRIVING_TITLE,
    });
    expect(harness.finalize).not.toHaveBeenCalled();

    deferred.resolve({ outcome: "imported", payload: importedVideo() });
    await capture;
    expect(harness.savePending).toHaveBeenCalledWith("space-1", {
      id: "media",
      expectedContent: expect.objectContaining({ processingState: "importing" }),
      data: importedVideo(),
    });
    expect(harness.finalize).toHaveBeenCalledWith("space-1", [expect.objectContaining({ id: "media" })], {
      reveal: true,
    });
    expect(harness.discard).not.toHaveBeenCalled();
  });

  it("reserves an audio node for a link that serves audio", async () => {
    mocks.allocateMediaImportAsset.mockResolvedValue("/spaces/assets/import-1.m4a");
    mocks.acquireMediaSource.mockResolvedValue({ outcome: "unsupported" });
    const harness = captureHarness();

    await expect(captureSpaceUrl(harness.context, "https://example.com/talk.mp3", POSITION)).rejects.toThrow(
      "unsupported",
    );

    expect(mocks.allocateMediaImportAsset).toHaveBeenCalledWith(expect.any(String), "audio");
    expect(harness.createPending).toHaveBeenCalledWith(
      "space-1",
      expect.objectContaining({ kind: "audio", assetPath: "/spaces/assets/import-1.m4a" }),
      POSITION,
    );
  });

  it("takes the reserved node away when the media source turns out to be unsupported", async () => {
    mocks.acquireMediaSource.mockResolvedValue({ outcome: "unsupported" });
    const harness = captureHarness();

    await expect(captureSpaceUrl(harness.context, URL, POSITION)).rejects.toThrow("unsupported");

    expect(harness.discard).toHaveBeenCalledWith("space-1", "media");
    expect(mocks.removeSpaceAssets).toHaveBeenCalledWith([RESERVED_ASSET]);
    expect(harness.savePending).not.toHaveBeenCalled();
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
    expect(harness.createPending).not.toHaveBeenCalled();
  });

  it("sends the reported YouTube watch URL to media acquisition, not website capture", async () => {
    const harness = captureHarness();
    const watchUrl = "https://www.youtube.com/watch?v=CmjNjWeRE0Y";
    mocks.acquireMediaSource.mockResolvedValue({ outcome: "imported", payload: importedVideo() });
    await captureSpaceUrl(harness.context, watchUrl, POSITION);
    expect(mocks.acquireMediaSource).toHaveBeenCalledWith(watchUrl, expect.any(Object));
    expect(mocks.addSavedWebsite).not.toHaveBeenCalled();
  });

  it("settles into its own Space after the person navigated away, without revealing it", async () => {
    const deferred = pendingImport();
    mocks.acquireMediaSource.mockReturnValue(deferred.promise);
    const harness = captureHarness();
    const capture = captureSpaceUrl(harness.context, URL, POSITION);
    await vi.waitFor(() => expect(harness.createPending).toHaveBeenCalledOnce());
    harness.setActiveSpace("space-2");
    deferred.resolve({ outcome: "imported", payload: importedVideo() });

    await capture;
    expect(harness.savePending).toHaveBeenCalledOnce();
    expect(harness.finalize).toHaveBeenCalledWith("space-1", [expect.objectContaining({ id: "media" })], {
      reveal: false,
    });
  });

  it("takes the arriving node and its asset away when the import fails, and reports why", async () => {
    const failure = new Error("runtime unavailable");
    mocks.acquireMediaSource.mockRejectedValue(failure);
    const harness = captureHarness();

    await expect(captureSpaceUrl(harness.context, URL, POSITION)).rejects.toBe(failure);

    expect(harness.discard).toHaveBeenCalledWith("space-1", "media");
    expect(mocks.removeSpaceAssets).toHaveBeenCalledWith([RESERVED_ASSET]);
    expect(harness.savePending).not.toHaveBeenCalled();
  });

  it("takes the arriving node away quietly when the import is cancelled", async () => {
    mocks.acquireMediaSource.mockRejectedValue(createAppError("media_acquisition_cancelled", "Cancelled"));
    const harness = captureHarness();
    await expect(captureSpaceUrl(harness.context, URL, POSITION)).resolves.toBeUndefined();
    expect(harness.discard).toHaveBeenCalledWith("space-1", "media");
    expect(mocks.logWarn).not.toHaveBeenCalled();
  });
});
