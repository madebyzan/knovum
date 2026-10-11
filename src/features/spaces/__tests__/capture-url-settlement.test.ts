import { beforeEach, expect, it, vi } from "vitest";

import type { MediaImportProgress } from "../media/import/schema/import";
import type { SpaceMediaPayload } from "../model/nodes/capabilities";
import { captureHarness, importedVideo, POSITION, URL } from "./support/capture-url-harness";

const mocks = vi.hoisted(() => ({
  acquireMediaSource:
    vi.fn<
      (
        url: string,
        onProgress: (progress: MediaImportProgress) => void,
      ) => Promise<{ outcome: "imported"; payload: SpaceMediaPayload } | { outcome: "unsupported" }>
    >(),
  addSavedWebsite: vi.fn<() => Promise<void>>(),
  logInfo: vi.fn<(message: string) => void>(),
  logWarn: vi.fn<(message: string) => void>(),
  removeSpaceAssets: vi.fn<(paths: readonly string[]) => Promise<void>>(),
  toastError: vi.fn<(message: string) => void>(),
}));

vi.mock("@tauri-apps/plugin-log", () => ({ info: mocks.logInfo, warn: mocks.logWarn }));
vi.mock("vue-sonner", () => ({ toast: { error: mocks.toastError } }));
vi.mock("../../../foundation/i18n/instance", () => ({
  i18n: { global: { t: (key: string) => key } },
}));
vi.mock("../assets/asset-operations", () => ({ removeSpaceAssets: mocks.removeSpaceAssets }));
vi.mock("../media/import/acquisition", () => ({ acquireMediaSource: mocks.acquireMediaSource }));
vi.mock("../website/capture", () => ({ addSavedWebsite: mocks.addSavedWebsite }));

import { captureSpaceUrl } from "../sources/capture-url";

beforeEach(() => {
  vi.clearAllMocks();
  mocks.addSavedWebsite.mockResolvedValue(undefined);
});

it("preserves persistence and downloaded asset cleanup failures", async () => {
  mocks.acquireMediaSource.mockResolvedValue({ outcome: "imported", payload: importedVideo() });
  mocks.removeSpaceAssets.mockRejectedValueOnce(new Error("asset locked"));
  const harness = captureHarness();
  harness.addContent.mockRejectedValue(new Error("revision rejected"));

  await captureSpaceUrl(harness.context, URL, POSITION);

  expect(mocks.logWarn).toHaveBeenCalledWith(expect.stringContaining("revision rejected"));
  expect(mocks.logWarn).toHaveBeenCalledWith(expect.stringContaining("asset locked"));
  expect(harness.update).toHaveBeenCalledWith("import-1", { status: "failed" });
});
