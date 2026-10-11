import { platform } from "@tauri-apps/plugin-os";
import { afterEach, describe, expect, it, vi } from "vitest";

import { isAssetPreviewAvailable } from "../asset-preview";

vi.mock("@tauri-apps/plugin-os", () => ({ platform: vi.fn<typeof platform>() }));
afterEach(() => vi.clearAllMocks());

describe("native asset preview availability", () => {
  it("enables the implemented macOS preview panel", () => {
    vi.mocked(platform).mockReturnValue("macos");
    expect(isAssetPreviewAvailable()).toBe(true);
  });
  it("enables the implemented Windows Shell preview host", () => {
    vi.mocked(platform).mockReturnValue("windows");
    expect(isAssetPreviewAvailable()).toBe(true);
  });
  it("omits preview on unsupported desktop platforms", () => {
    vi.mocked(platform).mockReturnValue("linux");
    expect(isAssetPreviewAvailable()).toBe(false);
  });
});
