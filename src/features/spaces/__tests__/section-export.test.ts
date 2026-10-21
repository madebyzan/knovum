import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { canvasNode, exportHost } from "./support/export-host";

vi.mock("@tauri-apps/api/core", async () => (await import("./support/export-host")).tauriCore);
vi.mock("@tauri-apps/plugin-dialog", async () => (await import("./support/export-host")).tauriDialog);

import { exportSpaceSection } from "../assets/export/section";

describe("Space section export", () => {
  beforeEach(() => {
    exportHost.invoke.mockReset().mockResolvedValue(undefined);
    exportHost.save.mockReset().mockResolvedValue("/exports/section.json");
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

    const call = exportHost.invoke.mock.calls[0]?.[1] as { text: string };
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
