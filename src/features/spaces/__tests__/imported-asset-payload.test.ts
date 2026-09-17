import { describe, expect, it } from "vitest";

import { importedSpaceNodePayload } from "../assets/imported-asset-payload";

describe("importedSpaceNodePayload", () => {
  it("accepts a persisted audio recording without a source revision", () => {
    expect(
      importedSpaceNodePayload({
        kind: "audio",
        name: "Audio 2026-08-22.webm",
        path: "/spaces/audio.webm",
      }),
    ).toEqual({
      kind: "audio",
      title: "Audio 2026-08-22.webm",
      fileName: "Audio 2026-08-22.webm",
      assetPath: "/spaces/audio.webm",
    });
  });

  it("preserves a source revision when the import provides one", () => {
    expect(
      importedSpaceNodePayload({
        kind: "video",
        name: "demo.mp4",
        path: "/spaces/demo.mp4",
        sourceRevision: "sha256:demo",
      }),
    ).toMatchObject({ sourceRevision: "sha256:demo" });
  });

  it("preserves intrinsic geometry for visual media and rendered documents", () => {
    expect(
      importedSpaceNodePayload({
        kind: "image",
        name: "portrait.jpg",
        path: "/spaces/portrait.jpg",
        intrinsicWidth: 1080,
        intrinsicHeight: 1920,
      }),
    ).toMatchObject({ kind: "image", intrinsicWidth: 1080, intrinsicHeight: 1920 });
    expect(
      importedSpaceNodePayload({
        kind: "file",
        name: "Quarterly.docx",
        path: "/spaces/Quarterly.docx",
        intrinsicWidth: 792,
        intrinsicHeight: 1024,
      }),
    ).toMatchObject({ kind: "file", intrinsicWidth: 792, intrinsicHeight: 1024 });
    expect(
      importedSpaceNodePayload({
        kind: "audio",
        name: "voice.webm",
        path: "/spaces/voice.webm",
        intrinsicWidth: 1080,
        intrinsicHeight: 1920,
      }),
    ).not.toHaveProperty("intrinsicWidth");
  });

  it("extracts leading H1 as title and strips it from markdown body", () => {
    expect(
      importedSpaceNodePayload({
        kind: "markdown",
        name: "draft.md",
        text: "# My Great Document\n\nFirst paragraph of content.\n\nSecond paragraph.",
      }),
    ).toEqual({
      kind: "markdown",
      title: "My Great Document",
      text: "First paragraph of content.\n\nSecond paragraph.",
    });
  });

  it("strips closing hashes and handles leading blank lines for markdown H1", () => {
    expect(
      importedSpaceNodePayload({
        kind: "markdown",
        name: "prompt.markdown",
        text: "\n\n# 科普图解生成Prompt ###\n\n【核心指令】\n内容详情",
      }),
    ).toEqual({
      kind: "markdown",
      title: "科普图解生成Prompt",
      text: "【核心指令】\n内容详情",
    });
  });

  it("preserves body and falls back to filename when markdown has no leading H1", () => {
    expect(
      importedSpaceNodePayload({
        kind: "markdown",
        name: "raw-notes.md",
        text: "## Section without H1\n\nRegular text here.",
      }),
    ).toEqual({
      kind: "markdown",
      title: "raw-notes",
      text: "## Section without H1\n\nRegular text here.",
    });
  });

  it("handles markdown containing only an H1 heading", () => {
    expect(
      importedSpaceNodePayload({
        kind: "markdown",
        name: "title-only.md",
        text: "# Only Title\n",
      }),
    ).toEqual({
      kind: "markdown",
      title: "Only Title",
      text: "",
    });
  });

  it("preserves archive entries for imported file assets", () => {
    const entries = [
      { path: "src/", kind: "folder" as const },
      { path: "src/index.ts", kind: "file" as const, size: 1024 },
    ];
    expect(
      importedSpaceNodePayload({
        kind: "file",
        name: "bundle.zip",
        path: "/spaces/bundle.zip",
        entries,
      }),
    ).toMatchObject({
      kind: "file",
      title: "bundle.zip",
      fileName: "bundle.zip",
      entries,
    });
  });
});
