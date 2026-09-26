import { beforeEach, describe, expect, it, vi } from "vitest";

import { deleteSpacePermanently } from "../workspace/space-operations";

const persistence = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: persistence.invoke,
}));

describe("Space deletion operations", () => {
  beforeEach(() => vi.clearAllMocks());

  it("returns private asset paths collected by the atomic deletion", async () => {
    persistence.invoke.mockResolvedValue([
      "/private/favicon.png",
      "/private/preview.webp",
      "/private/index.ts",
    ]);

    await expect(deleteSpacePermanently("space-1")).resolves.toEqual([
      "/private/favicon.png",
      "/private/preview.webp",
      "/private/index.ts",
    ]);
    expect(persistence.invoke).toHaveBeenCalledWith("plugin:spaces|delete_space_permanently", {
      request: { spaceId: "space-1" },
    });
  });
});
