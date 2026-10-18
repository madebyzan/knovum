import { beforeEach, expect, it, vi } from "vitest";

import type { MediaAcquisitionObserver } from "../media/import/acquisition";
import type { SpaceMediaPayload } from "../model/nodes/capabilities";
import {
  captureHarness,
  importedVideo,
  POSITION,
  RESERVED_ASSET,
  resolvedVideo,
  URL,
} from "./support/capture-url-harness";

const mocks = vi.hoisted(() => ({
  acquireMediaSource:
    vi.fn<
      (
        url: string,
        observer: MediaAcquisitionObserver,
      ) => Promise<{ outcome: "imported"; payload: SpaceMediaPayload } | { outcome: "unsupported" }>
    >(),
  addSavedWebsite: vi.fn<() => Promise<void>>(),
  logInfo: vi.fn<(message: string) => void>(),
  logWarn: vi.fn<(message: string) => void>(),
  removeSpaceAssets: vi.fn<(paths: readonly string[]) => Promise<void>>(),
}));

vi.mock("@tauri-apps/plugin-log", () => ({ info: mocks.logInfo, warn: mocks.logWarn }));
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

it("reports both the settlement failure and the asset cleanup that could not follow it", async () => {
  mocks.acquireMediaSource.mockImplementation(async (_url, observer) => {
    await observer.resolved(resolvedVideo());
    return { outcome: "imported", payload: importedVideo() };
  });
  mocks.removeSpaceAssets.mockRejectedValueOnce(new Error("asset locked"));
  const harness = captureHarness();
  harness.savePending.mockRejectedValue(new Error("revision rejected"));

  await expect(captureSpaceUrl(harness.context, URL, POSITION)).rejects.toThrow("revision rejected");

  expect(mocks.logWarn).toHaveBeenCalledWith(expect.stringContaining("revision rejected"));
  expect(mocks.logWarn).toHaveBeenCalledWith(expect.stringContaining("asset locked"));
  expect(harness.discard).toHaveBeenCalledWith("space-1", "media");
  expect(mocks.removeSpaceAssets).toHaveBeenCalledWith([RESERVED_ASSET]);
});
