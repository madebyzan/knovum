import { beforeEach, describe, expect, it, vi } from "vitest";

const clipboard = vi.hoisted(() => ({
  hasFiles: vi.fn<() => Promise<boolean>>(),
  readFiles: vi.fn<() => Promise<string[]>>(),
  readContent: vi.fn<() => Promise<{ html: string | null; text: string | null }>>(),
  importImage: vi.fn<() => Promise<{ kind: "image"; name: string; path: string } | undefined>>(),
  logInfo: vi.fn<(message: string) => Promise<void>>().mockResolvedValue(undefined),
  logWarn: vi.fn<(message: string) => Promise<void>>().mockResolvedValue(undefined),
}));

vi.mock("tauri-plugin-clipboard-api", () => ({
  hasFiles: clipboard.hasFiles,
  readFiles: clipboard.readFiles,
}));
vi.mock("../clipboard/image-import", () => ({ importSpaceClipboardImage: clipboard.importImage }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: clipboard.readContent }));
vi.mock("@tauri-apps/plugin-log", () => ({ info: clipboard.logInfo, warn: clipboard.logWarn }));

import { pasteSpaceClipboard } from "../clipboard/paste";

function handlers() {
  return {
    imageName: "Image",
    pasteNodes: vi.fn<(html: string | null) => Promise<boolean>>().mockResolvedValue(false),
    importFiles: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    addImage: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    insertText: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
  };
}

describe("canvas clipboard paste", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    clipboard.importImage.mockResolvedValue(undefined);
    clipboard.hasFiles.mockResolvedValue(false);
    clipboard.readContent.mockResolvedValue({ html: null, text: null });
  });

  it("keeps Knovum node copying ahead of system image and text content", async () => {
    const input = handlers();
    input.pasteNodes.mockResolvedValue(true);

    expect(await pasteSpaceClipboard(input)).toBe(true);
    expect(clipboard.importImage).not.toHaveBeenCalled();
    expect(clipboard.hasFiles).not.toHaveBeenCalled();
    expect(clipboard.readContent).toHaveBeenCalledOnce();
  });

  it("imports copied files through one batch regardless of media type", async () => {
    const input = handlers();
    clipboard.hasFiles.mockResolvedValue(true);
    clipboard.readFiles.mockResolvedValue(["/tmp/photo.png", "/tmp/movie.mp4", "/tmp/report.pdf"]);

    expect(await pasteSpaceClipboard(input)).toBe(true);
    expect(input.importFiles).toHaveBeenCalledWith(["/tmp/photo.png", "/tmp/movie.mp4", "/tmp/report.pdf"]);
    expect(clipboard.importImage).not.toHaveBeenCalled();
  });

  it("imports a screenshot image before any accompanying clipboard text", async () => {
    const input = handlers();
    const image = { kind: "image" as const, name: "Image", path: "/spaces/screenshot.png" };
    clipboard.importImage.mockResolvedValue(image);

    expect(await pasteSpaceClipboard(input)).toBe(true);
    expect(input.addImage).toHaveBeenCalledWith(image);
    expect(clipboard.readContent).toHaveBeenCalledOnce();
  });

  it.each([
    "https://www.youtube.com/watch?v=abc123",
    "https://www.bilibili.com/video/BV1example",
    "https://example.com/article",
  ])("routes clipboard URL %s through the existing text importer", async (url) => {
    const input = handlers();
    clipboard.readContent.mockResolvedValue({ html: null, text: url });

    expect(await pasteSpaceClipboard(input)).toBe(true);
    expect(input.insertText).toHaveBeenCalledWith(url);
  });

  it("does not let unrelated HTML prevent an accompanying URL from being pasted", async () => {
    const input = handlers();
    const url = "https://example.com/article";
    clipboard.readContent.mockResolvedValue({
      html: "<a href='https://example.com/article'>Article</a>",
      text: url,
    });

    expect(await pasteSpaceClipboard(input)).toBe(true);
    expect(input.pasteNodes).toHaveBeenCalledWith("<a href='https://example.com/article'>Article</a>");
    expect(input.insertText).toHaveBeenCalledWith(url);
  });

  it("returns normally when the clipboard has no supported content", async () => {
    const input = handlers();
    clipboard.readContent.mockResolvedValue({ html: null, text: null });

    expect(await pasteSpaceClipboard(input)).toBe(false);
    expect(input.insertText).not.toHaveBeenCalled();
  });

  it("records the failed stage without exposing clipboard text", async () => {
    const input = handlers();
    const secret = "https://example.com/private-token";
    clipboard.readContent.mockResolvedValue({ html: null, text: secret });
    input.insertText.mockRejectedValue(new Error(secret));

    await expect(pasteSpaceClipboard(input)).rejects.toThrow(secret);
    const messages = [...clipboard.logInfo.mock.calls, ...clipboard.logWarn.mock.calls].map(([message]) =>
      String(message),
    );
    expect(messages.join("\n")).not.toContain(secret);
    expect(messages.join("\n")).toContain("failed at insert-text");
  });
});
