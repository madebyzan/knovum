import { describe, expect, it } from "vitest";

import {
  exportJsonCanvasDocument,
  importJsonCanvasDocument,
  mergeImportedJsonCanvasDocument,
} from "../json-canvas-mapping";

describe("JSON Canvas 1.0 neutral mapping", () => {
  it("uses array order as the standard z-order and keeps Knovum fields out", () => {
    const document = exportJsonCanvasDocument(
      [
        {
          layout: {
            nodeId: "front",
            frame: { x: 1.4, y: 2.6, width: 320, height: 180, rotation: 15 },
            sizeAuthority: "frame",
            zIndex: 2,
          },
          content: { type: "text", text: "Front" },
        },
        {
          layout: {
            nodeId: "back",
            frame: { x: 0, y: 0, width: 640, height: 480, rotation: 0 },
            sizeAuthority: "frame",
            zIndex: 0,
            appearance: { fill: "pink", background: "assets/cover.png", backgroundStyle: "cover" },
          },
          content: { type: "group", label: "Back" },
        },
      ],
      [],
    );

    expect(document.nodes?.map(({ id }) => id)).toEqual(["back", "front"]);
    expect(document.nodes?.[1]).toEqual({
      id: "front",
      type: "text",
      text: "Front",
      x: 1,
      y: 3,
      width: 320,
      height: 180,
    });
    expect(document.nodes?.[0]).toMatchObject({
      color: "1",
      background: "assets/cover.png",
      backgroundStyle: "cover",
    });
  });

  it("preserves standard appearance and file subpaths outside domain content", () => {
    const plan = importJsonCanvasDocument({
      nodes: [
        {
          id: "file",
          type: "file",
          file: "notes.md",
          subpath: "#Heading",
          color: "3",
          x: 0,
          y: 0,
          width: 240,
          height: 120,
        },
      ],
    });

    expect(plan.nodes[0]).toMatchObject({
      appearance: { fill: "yellow" },
      fileSubpath: "#Heading",
      content: { type: "file", file: "notes.md" },
    });
    expect(
      exportJsonCanvasDocument(
        [{ layout: { ...plan.nodes[0]!, nodeId: "file" }, content: plan.nodes[0]!.content }],
        [],
      ).nodes?.[0],
    ).toMatchObject({ color: "3", subpath: "#Heading" });
  });

  it("round-trips standard custom colors without converting them to presets", () => {
    const plan = importJsonCanvasDocument({
      nodes: [
        {
          id: "custom",
          type: "text",
          text: "Custom",
          color: "#12aBcF",
          x: 0,
          y: 0,
          width: 240,
          height: 120,
        },
      ],
    });

    expect(plan.nodes[0]?.appearance).toEqual({ customFill: "#12aBcF" });
    expect(
      exportJsonCanvasDocument(
        [{ layout: { ...plan.nodes[0]!, nodeId: "custom" }, content: plan.nodes[0]!.content }],
        [],
      ).nodes?.[0],
    ).toMatchObject({ color: "#12aBcF" });
  });

  it("creates an engine-neutral import plan without inventing domain content", () => {
    const plan = importJsonCanvasDocument({
      nodes: [{ id: "n", type: "link", url: "https://example.com", x: 4, y: 8, width: 240, height: 120 }],
    });
    expect(plan.nodes).toEqual([
      {
        sourceId: "n",
        frame: { x: 4, y: 8, width: 240, height: 120, rotation: 0 },
        zIndex: 0,
        content: { type: "link", url: "https://example.com" },
      },
    ]);
  });

  it("assigns ordinary nodes to the smallest containing group without nesting groups", () => {
    const plan = importJsonCanvasDocument({
      nodes: [
        { id: "outer", type: "group", x: 0, y: 0, width: 500, height: 500 },
        { id: "inner", type: "group", x: 50, y: 50, width: 200, height: 200 },
        { id: "peer", type: "group", x: 50, y: 50, width: 200, height: 200 },
        { id: "note", type: "text", text: "Nested", x: 80, y: 80, width: 50, height: 50 },
      ],
    });

    expect(plan.nodes.map(({ sourceId, parentSourceId }) => [sourceId, parentSourceId])).toEqual([
      ["outer", undefined],
      ["inner", undefined],
      ["peer", undefined],
      ["note", "inner"],
    ]);
  });

  it("builds the complete persisted document from resolved imported node ids", () => {
    const plan = importJsonCanvasDocument({
      nodes: [
        { id: "a", type: "text", text: "A", x: 0, y: 0, width: 100, height: 80 },
        { id: "b", type: "text", text: "B", x: 120, y: 0, width: 100, height: 80 },
      ],
      edges: [{ id: "edge", fromNode: "a", toNode: "b", label: "supports" }],
    });
    const document = mergeImportedJsonCanvasDocument(
      {
        version: 1,
        viewport: { x: 8, y: 12, zoom: 1.5 },
        nodes: [
          {
            nodeId: "existing",
            frame: { x: 0, y: 0, width: 10, height: 10, rotation: 0 },
            sizeAuthority: "frame",
            zIndex: 4,
          },
        ],
      },
      {
        plan,
        importedNodes: [
          { sourceId: "a", nodeId: "node-a" },
          { sourceId: "b", nodeId: "node-b" },
        ],
      },
    );

    expect(document.viewport).toEqual({ x: 8, y: 12, zoom: 1.5 });
    expect(document.nodes.map(({ nodeId, zIndex }) => [nodeId, zIndex])).toEqual([
      ["existing", 4],
      ["node-a", 5],
      ["node-b", 6],
    ]);
    expect(document).not.toHaveProperty("connectors");
  });
});
