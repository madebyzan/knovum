import { beforeEach, describe, expect, it, vi } from "vitest";

import { createSpaceNode } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args: unknown) => Promise<unknown>>(),
  save: vi.fn<(options: unknown) => Promise<string | null>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: mocks.save }));

import { exportSpaceNode } from "../assets/export/node";

/** The formats the engine reports for the resource under test. */
function engineOffers(formats: string[]) {
  mocks.invoke
    .mockReset()
    .mockImplementation((command) =>
      Promise.resolve(command === "plugin:spaces|list_export_formats" ? formats : undefined),
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
      exportSpaceNode("space", canvasNode("credential", data as SpaceCanvasNode["data"])),
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

  it("offers every format the resource admits and saves the one the destination names", async () => {
    engineOffers(["txt", "html", "docx", "pdf", "pptx"]);
    mocks.save.mockResolvedValue("/exports/Meeting notes.docx");
    const onStart = vi.fn<(jobId: string) => void>();

    await exportSpaceNode("space", noteNode(), { onStart });

    expect(mocks.save).toHaveBeenCalledWith({
      defaultPath: "Meeting notes.txt",
      filters: [
        { name: "Plain text (.txt)", extensions: ["txt"] },
        { name: "HTML document (.html)", extensions: ["html"] },
        { name: "Word document (.docx)", extensions: ["docx"] },
        { name: "PDF document (.pdf)", extensions: ["pdf"] },
        { name: "Presentation (.pptx)", extensions: ["pptx"] },
      ],
    });
    expect(invocation("plugin:spaces|export_resource")).toEqual({
      spaceId: "space",
      nodeId: "note",
      destinationPath: "/exports/Meeting notes.docx",
      jobId: expect.any(String),
    });
    expect(onStart).toHaveBeenCalledOnce();
  });

  /** A markdown resource is the one case the Renderer and the engine both serve. */
  it("keeps its own format first and lets the Renderer write it", async () => {
    engineOffers(["txt", "html", "docx", "pdf", "pptx"]);
    mocks.save.mockResolvedValue("/exports/Plan.md");

    await exportSpaceNode(
      "space",
      canvasNode("markdown", { kind: "markdown", title: "Plan", text: "# Plan" }),
    );

    expect(mocks.save.mock.calls[0]?.[0]).toMatchObject({ defaultPath: "Plan.md" });
    expect(invocation("plugin:spaces|export_text")).toEqual({
      destinationPath: "/exports/Plan.md",
      text: "# Plan",
    });
    expect(invocation("plugin:spaces|export_resource")).toBeUndefined();
  });

  it("names a format that the dialog left unstated", async () => {
    engineOffers(["png", "jpg"]);
    mocks.save.mockResolvedValue("/exports/Cover");

    await exportSpaceNode("space", imageNode());

    expect(invocation("plugin:spaces|export_resource")).toMatchObject({
      destinationPath: "/exports/Cover.png",
    });
  });

  it("writes nothing when the destination is not chosen", async () => {
    engineOffers(["png", "jpg"]);
    mocks.save.mockResolvedValue(null);

    await expect(exportSpaceNode("space", imageNode())).resolves.toBe(false);

    expect(invocation("plugin:spaces|export_resource")).toBeUndefined();
    expect(invocation("plugin:spaces|export_text")).toBeUndefined();
  });

  it("refuses a resource that has no file of its own", async () => {
    engineOffers([]);
    await expect(exportSpaceNode("space", imageNode())).rejects.toThrow(
      "This resource has no exportable file",
    );
  });
});

function canvasNode(nodeId: string, data: SpaceCanvasNode["data"]): SpaceCanvasNode {
  return createSpaceNode(data, { x: 0, y: 0 }, nodeId);
}

function noteNode() {
  return canvasNode("note", {
    kind: "note",
    title: "Meeting notes",
    tiptapContent: {
      type: "doc",
      content: [{ type: "paragraph", content: [{ type: "text", text: "Decision" }] }],
    },
    plainText: "Decision",
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
