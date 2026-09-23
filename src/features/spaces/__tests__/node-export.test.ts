import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args: unknown) => Promise<unknown>>(),
  save: vi.fn<(options: unknown) => Promise<string | null>>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: mocks.save }));

import { exportSpaceNode, exportSpaceSection } from "../assets/node-export";

describe("Space node export", () => {
  beforeEach(() => {
    mocks.invoke.mockReset().mockResolvedValue(undefined);
    mocks.save.mockReset().mockResolvedValue("/exports/result");
  });

  it("exports subtitle nodes as correctly timed SubRip text", async () => {
    const node = canvasNode("subtitle", {
      kind: "subtitle",
      title: "Interview / captions",
      sourceVideoNodeId: "video",
      sourceVideoRevision: "revision",
      language: "en",
      durationMs: 3_700_001,
      cues: [
        { startMs: 1_234, endMs: 3_456, text: "First line" },
        { startMs: 3_600_000, endMs: 3_700_001, text: "Last line" },
      ],
    });

    await expect(exportSpaceNode(node)).resolves.toBe(true);

    expect(mocks.save).toHaveBeenCalledWith({
      defaultPath: "Interview - captions.srt",
      filters: [{ name: "SubRip subtitles", extensions: ["srt"] }],
    });
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|export_text", {
      destinationPath: "/exports/result",
      text: "1\n00:00:01,234 --> 00:00:03,456\nFirst line\n\n2\n01:00:00,000 --> 01:01:40,001\nLast line\n",
    });
  });

  it("exports credential metadata from an explicit safe allowlist", async () => {
    const data = {
      kind: "credential",
      title: "Publishing API",
      credentialKind: "api-key",
      credentialConfigured: true,
      secret: "must-not-leave-the-keychain",
      token: "must-not-leave-the-keychain",
    } as const;

    await exportSpaceNode(canvasNode("credential", data as SpaceCanvasNode["data"]));

    const call = mocks.invoke.mock.calls[0]?.[1] as { text: string };
    expect(JSON.parse(call.text)).toEqual({
      kind: "credential",
      title: "Publishing API",
      credentialKind: "api-key",
      configured: true,
    });
    expect(call.text).not.toContain(data.secret);
    expect(call.text).not.toContain(data.token);
  });

  it("exports a rich note as plain text without implying its formatting is preserved", async () => {
    await exportSpaceNode(
      canvasNode("note", {
        kind: "note",
        title: "Meeting notes",
        tiptapContent: {
          type: "doc",
          content: [{ type: "paragraph", content: [{ type: "text", text: "Decision" }] }],
        },
        plainText: "Decision",
      }),
    );

    expect(mocks.save).toHaveBeenCalledWith({
      defaultPath: "Meeting notes.txt",
      filters: [{ name: "Text", extensions: ["txt"] }],
    });
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|export_text", {
      destinationPath: "/exports/result",
      text: "Decision",
    });
  });

  it("delegates managed assets to the existing asset export command", async () => {
    const node = canvasNode("image", {
      kind: "image",
      title: "Cover",
      assetPath: "/managed/cover.png",
      fileName: "cover.png",
    });

    await exportSpaceNode(node);

    expect(mocks.save).toHaveBeenCalledWith({ defaultPath: "cover.png" });
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|export_asset", {
      sourcePath: "/managed/cover.png",
      destinationPath: "/exports/result",
    });
  });
});

describe("Space section export", () => {
  beforeEach(() => {
    mocks.invoke.mockReset().mockResolvedValue(undefined);
    mocks.save.mockReset().mockResolvedValue("/exports/section.json");
  });

  it("exports a section outline with credential metadata but no secrets", async () => {
    const credential = canvasNode("credential-node", {
      kind: "credential",
      title: "Deployment account",
      credentialKind: "api-key",
      credentialConfigured: true,
      secret: "never-export-this",
    } as unknown as SpaceCanvasNode["data"]);

    await exportSpaceSection(sectionSnapshot(), [credential], "section");

    const call = mocks.invoke.mock.calls[0]?.[1] as { text: string };
    expect(JSON.parse(call.text)).toEqual({
      version: 1,
      kind: "section",
      title: "Release sources",
      members: [
        {
          kind: "credential",
          title: "Deployment account",
          content: {
            kind: "credential",
            title: "Deployment account",
            credentialKind: "api-key",
            configured: true,
          },
        },
      ],
    });
    expect(call.text).not.toContain("never-export-this");
    expect(call.text).not.toContain("credentialConfigured");
  });
});

function canvasNode(id: string, data: SpaceCanvasNode["data"]): SpaceCanvasNode {
  return { id, type: data.kind, data, initialPosition: { x: 0, y: 0 } };
}

function sectionSnapshot(): SpaceEditSnapshot {
  return {
    editRevision: 1,
    relationStyles: [],
    structure: [
      { id: "section", kind: "section", parentId: null, order: 0, locked: false, title: "Release sources" },
      {
        id: "credential-structure",
        kind: "content",
        parentId: "section",
        order: 0,
        locked: false,
        nodeId: "credential-node",
      },
    ],
    layouts: [
      {
        structureNodeId: "section",
        kind: "section",
        position: { x: 0, y: 0 },
        size: { width: 400, height: 300 },
        collapsed: false,
      },
      {
        structureNodeId: "credential-structure",
        kind: "content",
        transform: { x: 20, y: 60, rotation: 0, scale: 1 },
        size: { width: 200, height: 120 },
        sizeAuthority: "frame",
      },
    ],
  };
}
