import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({ invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
import { decodeAppError } from "../../errors/app-error";
import { invokeCommand } from "../invoke-command";

beforeEach(() => {
  mocks.invoke.mockReset();
});

describe("application IPC error boundary", () => {
  it("returns the IPC response without replacing it", async () => {
    const nodeProjection = { nodeId: "node-id" };
    mocks.invoke.mockResolvedValue(nodeProjection);
    await expect(invokeCommand<typeof nodeProjection>("plugin:spaces|get_node")).resolves.toBe(
      nodeProjection,
    );
    expect(mocks.invoke).toHaveBeenCalledExactlyOnceWith("plugin:spaces|get_node");
  });
  it("preserves arguments and invoke options without adding omitted parameters", async () => {
    const args = { jobId: "job-id", onEvent: { id: 12 } };
    const options = { headers: { "X-Request-ID": "request-id" } };
    mocks.invoke.mockResolvedValue(undefined);
    await invokeCommand<void>("plugin:ai|validate_model_settings", args);
    expect(mocks.invoke.mock.calls[0]).toEqual(["plugin:ai|validate_model_settings", args]);
    expect(mocks.invoke.mock.calls[0]?.[1]).toBe(args);
    await invokeCommand<void>("plugin:ai|validate_model_settings", args, options);
    expect(mocks.invoke.mock.calls[1]).toEqual(["plugin:ai|validate_model_settings", args, options]);
    expect(mocks.invoke.mock.calls[1]?.[2]).toBe(options);
    await invokeCommand<void>("plugin:ai|validate_model_settings", undefined, options);
    expect(mocks.invoke.mock.calls[2]).toEqual(["plugin:ai|validate_model_settings", undefined, options]);
  });
  it("turns a IPC code/message rejection into an Error while preserving its machine code", async () => {
    const payload = { code: "conversation_upload_size", message: "Attachment exceeds the limit" };
    mocks.invoke.mockRejectedValue(payload);
    const error = await invokeCommand("plugin:spaces|inspect_conversation_upload", {
      sourcePath: "/tmp/file",
    }).catch((reason: unknown) => reason);
    expect(error).toBeInstanceOf(Error);
    expect(decodeAppError(error)).toEqual(payload);
  });
  it("preserves errors that are already normalized", async () => {
    const error = new Error("Cancelled", { cause: { code: "cancelled", message: "Cancelled" } });
    mocks.invoke.mockRejectedValue(error);
    await expect(invokeCommand("plugin:ai|validate_model_settings")).rejects.toBe(error);
  });
  it("reports Tauri argument/permission rejections as visible errors", async () => {
    mocks.invoke.mockRejectedValue("Command not permitted");
    await expect(invokeCommand("plugin:spaces|remove_assets")).rejects.toThrow("Command not permitted");
  });
  it.each([
    { code: "", message: "Invalid code" },
    { code: "operation_failed", message: "Unknown fields", obsolete: true },
    { code: "operation_failed" },
  ])("does not reinterpret malformed command errors: %j", async (payload) => {
    mocks.invoke.mockRejectedValue(payload);
    const error = await invokeCommand<void>("plugin:spaces|get_node").catch((reason: unknown) => reason);
    expect(error).toBeInstanceOf(Error);
    expect(error).toHaveProperty("cause", payload);
    expect(decodeAppError(error)).toBeNull();
  });
});
