import { describe, expect, it } from "vitest";

import { parseCanvasDocument, serializeCanvasAppearance } from "../canvas/contract/canvas-document";

function document() {
  return {
    version: 1,
    viewport: { x: 12, y: -8, zoom: 1.25 },
    nodes: [
      {
        nodeId: "section-1",
        frame: { x: 0, y: 0, width: 640, height: 480, rotation: 0 },
        sizeAuthority: "frame",
        zIndex: 0,
      },
      {
        nodeId: "note-1",
        frame: { x: 32, y: 48, width: 320, height: 180, rotation: 0 },
        sizeAuthority: "frame",
        appearance: { fill: "pink" },
        fileSubpath: "#Heading",
        zIndex: 1,
      },
    ],
  } as const;
}

describe("CanvasDocument v1", () => {
  it("accepts engine-neutral layout state without domain content", () => {
    expect(parseCanvasDocument(document())).toEqual(document());
  });

  it("rejects engine snapshots and copied domain content", () => {
    expect(() => parseCanvasDocument({ ...document(), store: {} })).toThrow(/unrecognized_keys/u);
    expect(() =>
      parseCanvasDocument({
        ...document(),
        nodes: [{ ...document().nodes[0], title: "copied domain content" }],
      }),
    ).toThrow(/unrecognized_keys/u);
  });

  it("rejects duplicate nodes", () => {
    const node = document().nodes[0];
    expect(() => parseCanvasDocument({ ...document(), nodes: [node, node] })).toThrow(/unique/u);
  });

  it("requires explicit size ownership instead of inferring it from dimensions", () => {
    const { sizeAuthority: _sizeAuthority, ...node } = document().nodes[0];
    expect(() => parseCanvasDocument({ ...document(), nodes: [node] })).toThrow(/./u);
  });

  it("uses the same unsigned 32-bit layer range as the Rust contract", () => {
    expect(
      parseCanvasDocument({
        ...document(),
        nodes: [{ ...document().nodes[0], zIndex: 0xffff_ffff }],
      }).nodes[0]?.zIndex,
    ).toBe(0xffff_ffff);
    expect(() =>
      parseCanvasDocument({
        ...document(),
        nodes: [{ ...document().nodes[0], zIndex: 0x1_0000_0000 }],
      }),
    ).toThrow(/4294967295|too_big|less_than_equal/u);
  });

  it("rejects duplicated domain membership and lock state", () => {
    expect(() =>
      parseCanvasDocument({
        ...document(),
        nodes: [{ ...document().nodes[0], parentNodeId: "section", locked: true }],
      }),
    ).toThrow(/unrecognized_keys/u);
  });

  it("accepts only the current semantic appearance contract", () => {
    expect(
      parseCanvasDocument({
        ...document(),
        nodes: [
          {
            ...document().nodes[0],
            appearance: {
              customFill: "#12aBcF",
              fontStyle: "italic",
              textDecoration: "underline",
              typographyScale: 1.5,
              verticalAlign: "middle",
            },
          },
        ],
      }).nodes[0]?.appearance,
    ).toEqual({
      customFill: "#12aBcF",
      fontStyle: "italic",
      textDecoration: "underline",
      typographyScale: 1.5,
      verticalAlign: "middle",
    });
    expect(() =>
      parseCanvasDocument({
        ...document(),
        nodes: [{ ...document().nodes[0], appearance: { typographyScale: 4.01 } }],
      }),
    ).toThrow(/./u);
    expect(() =>
      parseCanvasDocument({
        ...document(),
        nodes: [{ ...document().nodes[0], appearance: { color: "2" } }],
      }),
    ).toThrow(/unrecognized_keys/u);
    expect(() =>
      parseCanvasDocument({
        ...document(),
        nodes: [{ ...document().nodes[0], appearance: { fontSize: 576.1 } }],
      }),
    ).toThrow(/./u);
    expect(() =>
      parseCanvasDocument({
        ...document(),
        nodes: [{ ...document().nodes[0], appearance: { customFill: "red" } }],
      }),
    ).toThrow(/./u);
    expect(() =>
      parseCanvasDocument({
        ...document(),
        nodes: [
          {
            ...document().nodes[0],
            appearance: { fill: "yellow", customFill: "#123456" },
          },
        ],
      }),
    ).toThrow(/mutually exclusive/u);
  });

  it("rejects semantic relations embedded in the spatial document", () => {
    expect(() => parseCanvasDocument({ ...document(), connectors: [] })).toThrow(/unrecognized_keys/u);
  });

  it("stores only non-default relation presentation overrides", () => {
    expect(
      parseCanvasDocument({
        ...document(),
        relationStyles: [{ relationId: "relation", color: "blue" }],
      }).relationStyles,
    ).toEqual([{ relationId: "relation", color: "blue" }]);
    expect(() =>
      parseCanvasDocument({
        ...document(),
        relationStyles: [{ relationId: "relation", color: "neutral" }],
      }),
    ).toThrow(/./u);
  });
});

it("serializes appearance without undefined fields", () => {
  const appearance = { fill: "yellow" as const, fontSize: 16, background: undefined };
  expect(serializeCanvasAppearance(appearance)).toEqual({ fill: "yellow", fontSize: 16 });
  expect(appearance).toHaveProperty("background", undefined);
});
