import { describe, expect, it, vi } from "vitest";

import { persistTiptapSession } from "../editor/tiptap-session-participant";

describe("Tiptap editor close", () => {
  it("keeps the editor loaded when persistence fails", async () => {
    const failure = new Error("disk full");
    const leave = vi.fn<() => void>();

    await expect(
      persistTiptapSession(vi.fn<() => Promise<void>>().mockRejectedValue(failure), leave),
    ).rejects.toBe(failure);

    expect(leave).not.toHaveBeenCalled();
  });

  it("leaves only after persistence succeeds", async () => {
    const leave = vi.fn<() => void>();

    await persistTiptapSession(vi.fn<() => Promise<void>>().mockResolvedValue(undefined), leave);

    expect(leave).toHaveBeenCalledOnce();
  });
});
