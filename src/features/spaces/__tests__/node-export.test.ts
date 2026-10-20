import { beforeEach, describe, expect, it, vi } from "vitest";

import { createSpaceNode } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args: unknown) => Promise<unknown>>(),
  save: vi.fn<(options: unknown) => Promise<string | null>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: mocks.save }));

import { nodeExportFormats } from "../assets/export/formats";
import { exportSpaceNode } from "../assets/export/node";

/** The formats the engine reports for the resource under test. */
function engineOffers(formats: string[], unavailable: string | null = null) {
  mocks.invoke
    .mockReset()
    .mockImplementation((command) =>
      Promise.resolve(command === "plugin:spaces|list_export_formats" ? { formats, unavailable } : undefined),
    );
}

function invocation(command: string) {
  return mocks.invoke.mock.calls.find(([name]) => name === command)?.[1];
}

describe("Space resource export", () => {
  beforeEach(() => {
    engineOffers([]);
    mocks.save.mockReset().mockResolvedValue("/exports/result.json");
  });

  it("serializes credential metadata from an explicit allowlist, without the engine", async () => {
    const data = {
      kind: "credential",
      title: "Publishing API",
      credentialKind: "api-key",
      credentialConfigured: true,
      secret: "must-not-leave-the-keychain",
      token: "must-not-leave-the-keychain",
    } as const;

    await expect(
      exportSpaceNode("space", canvasNode("credential", data as SpaceCanvasNode["data"]), "json"),
    ).resolves.toBe(true);

    const call = invocation("plugin:spaces|export_text") as { text: string };
    expect(JSON.parse(call.text)).toEqual({
      kind: "credential",
      title: "Publishing API",
      credentialKind: "api-key",
      configured: true,
    });
    expect(call.text).not.toContain(data.secret);
    expect(call.text).not.toContain(data.token);
  });

  /** A markdown resource is the one case the Renderer and the engine both serve. */
  it("admits its own format first, then the ones the engine converts to", async () => {
    engineOffers(["txt", "html", "docx", "pdf", "pptx"]);

    await expect(nodeExportFormats("space", canvasNode("markdown", markdownPayload()))).resolves.toEqual({
      formats: ["md", "txt", "html", "docx", "pdf", "pptx"],
      unavailable: null,
    });
  });

  it("admits nothing for a resource that has no file of its own", async () => {
    engineOffers([]);

    await expect(nodeExportFormats("space", imageNode())).resolves.toEqual({
      formats: [],
      unavailable: null,
    });
  });

  /** Every document format reads the same content, so one unreadable document rules them all out. */
  it("keeps the format the Renderer holds and carries the engine's reason for the rest", async () => {
    engineOffers([], "Document export input contains unsupported control characters");

    await expect(nodeExportFormats("space", canvasNode("markdown", markdownPayload()))).resolves.toEqual({
      formats: ["md"],
      unavailable: "Document export input contains unsupported control characters",
    });
  });

  /** Content the engine reads is the engine's to serialize, so the Renderer adds nothing to it. */
  it("leaves the formats of content the engine reads to the engine", async () => {
    engineOffers(["txt", "html", "docx", "pdf", "pptx"]);

    const formats = await Promise.all(
      [editableNode("text", "Notes"), captureNode()].map((node) => nodeExportFormats("space", node)),
    );

    expect(formats).toEqual([
      { formats: ["txt", "html", "docx", "pdf", "pptx"], unavailable: null },
      { formats: ["txt", "html", "docx", "pdf", "pptx"], unavailable: null },
    ]);
  });

  it("asks for a destination in the chosen format and lets the engine write it", async () => {
    mocks.save.mockResolvedValue("/exports/Meeting notes.docx");
    const onStart = vi.fn<(jobId: string) => void>();

    await exportSpaceNode("space", editableNode("note", "Meeting notes"), "docx", { onStart });

    expect(mocks.save).toHaveBeenCalledWith({
      defaultPath: "Meeting notes.docx",
      filters: [{ name: "Word document (.docx)", extensions: ["docx"] }],
    });
    expect(invocation("plugin:spaces|export_resource")).toEqual({
      spaceId: "space",
      nodeId: "note",
      destinationPath: "/exports/Meeting notes.docx",
      jobId: expect.any(String),
    });
    expect(onStart).toHaveBeenCalledOnce();
  });

  it("lets the Renderer write a format it holds itself", async () => {
    mocks.save.mockResolvedValue("/exports/Plan.md");

    await exportSpaceNode("space", canvasNode("markdown", markdownPayload()), "md");

    expect(invocation("plugin:spaces|export_text")).toEqual({
      destinationPath: "/exports/Plan.md",
      text: "# Plan",
    });
    expect(invocation("plugin:spaces|export_resource")).toBeUndefined();
  });

  it("names the chosen format on a destination that states another one", async () => {
    mocks.save.mockResolvedValue("/exports/Cover.txt");

    await exportSpaceNode("space", imageNode(), "png");

    expect(invocation("plugin:spaces|export_resource")).toMatchObject({
      destinationPath: "/exports/Cover.txt.png",
    });
  });

  it("writes nothing when the destination is not chosen", async () => {
    mocks.save.mockResolvedValue(null);

    await expect(exportSpaceNode("space", imageNode(), "png")).resolves.toBe(false);

    expect(invocation("plugin:spaces|export_resource")).toBeUndefined();
    expect(invocation("plugin:spaces|export_text")).toBeUndefined();
  });
});

function canvasNode(nodeId: string, data: SpaceCanvasNode["data"]): SpaceCanvasNode {
  return createSpaceNode(data, { x: 0, y: 0 }, nodeId);
}

function markdownPayload(): SpaceCanvasNode["data"] {
  return { kind: "markdown", title: "Plan", text: "# Plan" };
}

/** A Note and a text block carry the same editable content, so one fixture describes both. */
function editableNode(kind: "note" | "text", title: string) {
  return canvasNode(kind, {
    kind,
    title,
    tiptapContent: {
      type: "doc",
      content: [{ type: "paragraph", content: [{ type: "text", text: "Decision" }] }],
    },
    plainText: "Decision",
  });
}

/** A capture has no file: the engine assembles the summary it holds and every format of it. */
function captureNode() {
  return canvasNode("website", {
    kind: "website",
    title: "Release notes",
    url: "https://example.com/notes",
  });
}

function imageNode() {
  return canvasNode("image", {
    kind: "image",
    title: "Cover",
    assetPath: "/managed/cover.png",
    fileName: "cover.png",
  });
}
