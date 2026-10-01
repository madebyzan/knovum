import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";

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

  it("exports credential metadata from an explicit safe allowlist", async () => {
    const data = {
      kind: "credential",
      title: "Publishing API",
      credentialKind: "api-key",
      credentialConfigured: true,
      secret: "must-not-leave-the-keychain",
      token: "must-not-leave-the-keychain",
    } as const;

    await exportSpaceNode("space", canvasNode("credential", data as SpaceCanvasNode["data"]));

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
      "space",
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

  it("exports image nodes through the Rust canonical renderer", async () => {
    const node = canvasNode("image", {
      kind: "image",
      title: "Cover",
      assetPath: "/managed/cover.png",
      fileName: "cover.png",
    });

    await exportSpaceNode("space", node);

    expect(mocks.save).toHaveBeenCalledWith({
      defaultPath: "cover.png",
      filters: [{ name: "PNG", extensions: ["png"] }],
    });
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:spaces|export_image", {
      spaceId: "space",
      nodeId: "image",
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
        sizingMode: "frame",
      },
    ],
  };
}
