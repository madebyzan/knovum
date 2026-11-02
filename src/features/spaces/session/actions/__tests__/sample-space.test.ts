import { beforeEach, describe, expect, it, vi } from "vitest";

import { invokeCommand } from "../../../../../foundation/ipc/invoke-command";
import { createAndTrackSpace } from "../../../workspace/space-creation";
import type { SpaceSessionContext } from "../../../workspace/space-session-context";
import { openSpace } from "../../../workspace/workbench-navigation-actions";
import type { SpaceState } from "../../session-state";
import { seedSampleSpace } from "../sample-space";

vi.mock("../../../../../foundation/ipc/invoke-command", () => ({
  invokeCommand: vi.fn<() => Promise<string>>(),
}));
vi.mock("../../../workspace/space-creation", () => ({
  createAndTrackSpace: vi.fn<() => Promise<unknown>>(),
}));
vi.mock("../../../workspace/workbench-navigation-actions", () => ({
  openSpace: vi.fn<() => Promise<unknown>>(),
}));

describe("seedSampleSpace", () => {
  beforeEach(() => {
    vi.mocked(invokeCommand).mockResolvedValue("/resources/sample-space/en.canvas");
    vi.mocked(createAndTrackSpace).mockResolvedValue({ id: "sample" } as never);
  });

  it("imports the bundled document into a new Space it has just opened", async () => {
    const order: string[] = [];
    vi.mocked(openSpace).mockImplementation(async () => {
      order.push("open");
      return true;
    });
    const importJsonCanvas = vi.fn<(request: unknown) => Promise<never>>(async () => {
      order.push("import");
      return undefined as never;
    });
    const state = { editSnapshot: { value: { editRevision: 7 } } } as unknown as SpaceState;

    await seedSampleSpace({} as SpaceSessionContext, state, importJsonCanvas);

    expect(createAndTrackSpace).toHaveBeenCalledWith({}, expect.any(String), "space");
    expect(order).toEqual(["open", "import"]);
    expect(importJsonCanvas).toHaveBeenCalledWith({
      sourcePath: "/resources/sample-space/en.canvas",
      baseRevision: 7,
    });
  });
});
