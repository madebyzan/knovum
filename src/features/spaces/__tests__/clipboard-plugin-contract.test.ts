import { emit } from "@tauri-apps/api/event";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { onFilesUpdate, onImageBinaryUpdate, onTextUpdate } from "tauri-plugin-clipboard-api";
import { afterEach, describe, expect, it, vi } from "vitest";

afterEach(() => clearMocks());

describe("clipboard validation after the Valibot security update", () => {
  it("accepts native payloads and rejects malformed payloads", async () => {
    mockIPC(() => undefined, { shouldMockEvents: true });
    const text = vi.fn<(value: string) => void>();
    const files = vi.fn<(value: string[]) => void>();
    const image = vi.fn<(value: number[]) => void>();
    await onTextUpdate(text);
    await onFilesUpdate(files);
    await onImageBinaryUpdate(image);
    await emit("plugin:clipboard://text-changed", { value: "hello" });
    await emit("plugin:clipboard://files-changed", { value: ["/tmp/example.png"] });
    await emit("plugin:clipboard://image-changed-binary", { value: [1, 2, 3] });
    expect(text).toHaveBeenCalledWith("hello");
    expect(files).toHaveBeenCalledWith(["/tmp/example.png"]);
    expect(image).toHaveBeenCalledWith([1, 2, 3]);
    await expect(emit("plugin:clipboard://text-changed", { value: 42 })).rejects.toThrow(/Invalid type/);
    await expect(emit("plugin:clipboard://files-changed", { value: [42] })).rejects.toThrow(/Invalid type/);
    expect(text).toHaveBeenCalledOnce();
  });
});
