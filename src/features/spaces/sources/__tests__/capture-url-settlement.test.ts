import { beforeEach, expect, it, vi } from "vitest";

import {
  captureHarness,
  importedVideo,
  POSITION,
  RESERVED_ASSET,
  URL,
} from "../../__tests__/support/capture-url-harness";
import type { MediaAcquisitionSubject } from "../../media/import/acquisition";
import type { SpaceMediaPayload } from "../../model/nodes/capabilities";

const mocks = vi.hoisted(() => ({
  acquireMediaSource:
    vi.fn<
      (
        url: string,
        subject: MediaAcquisitionSubject,
      ) => Promise<{ outcome: "imported"; payload: SpaceMediaPayload } | { outcome: "unsupported" }>
    >(),
  allocateMediaImportAsset: vi.fn<(jobId: string, kind: "audio" | "video") => Promise<string>>(),
  addSavedWebsite: vi.fn<() => Promise<void>>(),
  logInfo: vi.fn<(message: string) => void>(),
  logWarn: vi.fn<(message: string) => void>(),
  removeSpaceAssets: vi.fn<(paths: readonly string[]) => Promise<void>>(),
}));

vi.mock("@tauri-apps/plugin-log", () => ({ info: mocks.logInfo, warn: mocks.logWarn }));
vi.mock("../../../../foundation/i18n/instance", () => ({
  i18n: { global: { t: (key: string) => key } },
}));
vi.mock("../../assets/asset-operations", () => ({ removeSpaceAssets: mocks.removeSpaceAssets }));
vi.mock("../../media/import/acquisition", () => ({ acquireMediaSource: mocks.acquireMediaSource }));
vi.mock("../../media/import/ipc", () => ({ allocateMediaImportAsset: mocks.allocateMediaImportAsset }));
vi.mock("../../website/capture", () => ({ addSavedWebsite: mocks.addSavedWebsite }));

import { captureSpaceUrl } from "../capture-url";

beforeEach(() => {
  vi.clearAllMocks();
  mocks.addSavedWebsite.mockResolvedValue(undefined);
  mocks.allocateMediaImportAsset.mockResolvedValue(RESERVED_ASSET);
});

it("reports both the settlement failure and the asset cleanup that could not follow it", async () => {
  mocks.acquireMediaSource.mockResolvedValue({ outcome: "imported", payload: importedVideo() });
  mocks.removeSpaceAssets.mockRejectedValueOnce(new Error("asset locked"));
  const harness = captureHarness();
  harness.savePending.mockRejectedValue(new Error("revision rejected"));

  await expect(captureSpaceUrl(harness.context, URL, POSITION)).rejects.toThrow("revision rejected");

  expect(mocks.logWarn).toHaveBeenCalledWith(expect.stringContaining("revision rejected"));
  expect(mocks.logWarn).toHaveBeenCalledWith(expect.stringContaining("asset locked"));
  expect(harness.discard).toHaveBeenCalledWith("space-1", "media");
  expect(mocks.removeSpaceAssets).toHaveBeenCalledWith([RESERVED_ASSET]);
});

it("refuses an import that published somewhere other than the reserved asset", async () => {
  mocks.acquireMediaSource.mockResolvedValue({
    outcome: "imported",
    payload: { ...importedVideo(), assetPath: "/spaces/assets/elsewhere.mp4" },
  });
  const harness = captureHarness();

  await expect(captureSpaceUrl(harness.context, URL, POSITION)).rejects.toThrow("reserved asset");

  expect(harness.savePending).not.toHaveBeenCalled();
  expect(harness.discard).toHaveBeenCalledWith("space-1", "media");
});

it("refuses an import that published another kind of media than the link asked for", async () => {
  mocks.acquireMediaSource.mockResolvedValue({
    outcome: "imported",
    payload: { ...importedVideo(), kind: "audio" },
  });
  const harness = captureHarness();

  await expect(captureSpaceUrl(harness.context, URL, POSITION)).rejects.toThrow(
    "not the kind its link asked for",
  );

  expect(harness.savePending).not.toHaveBeenCalled();
  expect(harness.discard).toHaveBeenCalledWith("space-1", "media");
});
