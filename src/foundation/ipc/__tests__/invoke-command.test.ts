import { beforeEach, describe, expect, it, vi } from "vitest";

const native = vi.hoisted(() => ({ invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: native.invoke }));
import { decodeCommandError } from "../command-error";
import { invokeCommand } from "../invoke-command";

beforeEach(() => {
  native.invoke.mockReset();
});

describe("application IPC error boundary", () => {
  it("turns a native code/message rejection into an Error while preserving its machine code", async () => {
    const payload = { code: "assistant_upload_size", message: "Attachment exceeds the limit" };
    native.invoke.mockRejectedValue(payload);
    const error = await invokeCommand("plugin:spaces|inspect_assistant_upload", {
      sourcePath: "/tmp/file",
    }).catch((reason: unknown) => reason);
    expect(error).toBeInstanceOf(Error);
    expect(decodeCommandError(error)).toEqual(payload);
  });
  it("preserves errors that are already normalized", async () => {
    const error = new Error("Cancelled", { cause: { code: "cancelled", message: "Cancelled" } });
    native.invoke.mockRejectedValue(error);
    await expect(invokeCommand("plugin:ai|generate_text")).rejects.toBe(error);
  });
  it("reports Tauri argument/permission rejections as visible errors", async () => {
    native.invoke.mockRejectedValue("Command not permitted");
    await expect(invokeCommand("plugin:spaces|remove_assets")).rejects.toThrow("Command not permitted");
  });
});
