import { beforeEach, describe, expect, it, vi } from "vitest";

import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import { prepareVideoEditProxy } from "../prepare-video-edit-proxy";

vi.mock("../../../../foundation/ipc/invoke-command", () => ({
  invokeCommand: vi.fn<typeof invokeCommand>(),
}));
vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));
const source = { spaceId: "space", sourceNodeId: "video", sourceRevision: "revision" };
beforeEach(() => vi.clearAllMocks());

describe("canonical editing proxy preparation", () => {
  it("admits a validated prepared path", async () => {
    vi.mocked(invokeCommand).mockResolvedValueOnce("/cache/proxy.mp4");
    await expect(prepareVideoEditProxy(source, new AbortController().signal)).resolves.toBe(
      "/cache/proxy.mp4",
    );
    expect(invokeCommand).toHaveBeenCalledWith("plugin:spaces|prepare_video_edit_proxy", {
      request: { id: expect.any(String), source },
    });
  });
  it("cancels the native job and rejects a result delivered after disposal", async () => {
    let complete!: (path: string) => void;
    vi.mocked(invokeCommand).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          complete = resolve;
        }),
    );
    vi.mocked(invokeCommand).mockResolvedValue(undefined);
    const controller = new AbortController();
    const pending = prepareVideoEditProxy(source, controller.signal);
    const rejected = pending.catch((error: unknown) => error);
    controller.abort();
    complete("/cache/late.mp4");
    expect(await rejected).toMatchObject({ name: "AbortError" });
    expect(invokeCommand).toHaveBeenLastCalledWith("plugin:spaces|cancel_video_edit_proxy", {
      jobId: expect.any(String),
    });
  });
  it("does not start a job for an already disposed editor", async () => {
    const controller = new AbortController();
    controller.abort();
    await expect(prepareVideoEditProxy(source, controller.signal)).rejects.toMatchObject({
      name: "AbortError",
    });
    expect(invokeCommand).not.toHaveBeenCalled();
  });
  it("rejects an empty native path", async () => {
    vi.mocked(invokeCommand).mockResolvedValueOnce("");
    await expect(prepareVideoEditProxy(source, new AbortController().signal)).rejects.toMatchObject({
      name: "ZodError",
    });
  });
});
