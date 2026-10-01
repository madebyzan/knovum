import { beforeEach, describe, expect, it, vi } from "vitest";

const native = vi.hoisted(() => ({ invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: native.invoke }));
import { decodeAppError } from "../../errors/app-error";
import { invokeCommand } from "../invoke-command";

beforeEach(() => {
  native.invoke.mockReset();
});

describe("application IPC error boundary", () => {
  it("returns the native result without replacing it", async () => {
    const result = { nodeId: "node-id" };
    native.invoke.mockResolvedValue(result);
    await expect(invokeCommand<typeof result>("plugin:spaces|get_node")).resolves.toBe(result);
    expect(native.invoke).toHaveBeenCalledExactlyOnceWith("plugin:spaces|get_node");
  });
  it("preserves arguments and native options without adding omitted parameters", async () => {
    const args = { jobId: "job-id", onEvent: { id: 12 } };
    const options = { headers: { "X-Request-ID": "request-id" } };
    native.invoke.mockResolvedValue(undefined);
    await invokeCommand<void>("plugin:ai|generate_text", args);
    expect(native.invoke.mock.calls[0]).toEqual(["plugin:ai|generate_text", args]);
    expect(native.invoke.mock.calls[0]?.[1]).toBe(args);
    await invokeCommand<void>("plugin:ai|generate_text", args, options);
    expect(native.invoke.mock.calls[1]).toEqual(["plugin:ai|generate_text", args, options]);
    expect(native.invoke.mock.calls[1]?.[2]).toBe(options);
    await invokeCommand<void>("plugin:ai|generate_text", undefined, options);
    expect(native.invoke.mock.calls[2]).toEqual(["plugin:ai|generate_text", undefined, options]);
  });
  it("turns a native code/message rejection into an Error while preserving its machine code", async () => {
    const payload = { code: "assistant_upload_size", message: "Attachment exceeds the limit" };
    native.invoke.mockRejectedValue(payload);
    const error = await invokeCommand("plugin:spaces|inspect_assistant_upload", {
      sourcePath: "/tmp/file",
    }).catch((reason: unknown) => reason);
    expect(error).toBeInstanceOf(Error);
    expect(decodeAppError(error)).toEqual(payload);
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
  it.each([
    { code: "", message: "Invalid code" },
    { code: "operation_failed", message: "Unknown fields", obsolete: true },
    { code: "operation_failed" },
  ])("does not reinterpret malformed command errors: %j", async (payload) => {
    native.invoke.mockRejectedValue(payload);
    const error = await invokeCommand<void>("plugin:spaces|get_node").catch((reason: unknown) => reason);
    expect(error).toBeInstanceOf(Error);
    expect(error).toHaveProperty("cause", payload);
    expect(decodeAppError(error)).toBeNull();
  });
});
