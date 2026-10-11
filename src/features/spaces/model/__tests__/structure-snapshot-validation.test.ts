import { describe, expect, it } from "vitest";

import { parseSpaceEditSnapshot, type SpaceEditSnapshot } from "../structure/schema/edit-snapshot";
import { MAX_WORLD_COORDINATE } from "../structure/schema/structure";

function snapshot(): SpaceEditSnapshot {
  return {
    editRevision: 0,
    relationStyles: [],
    structure: [
      { id: "section", kind: "section", parentId: null, order: 0, locked: false, title: "Section" },
      { id: "content", kind: "content", parentId: "section", order: 0, locked: false, nodeId: "node" },
    ],
    layouts: [
      {
        kind: "section",
        structureNodeId: "section",
        position: { x: 0, y: 0 },
        size: { width: 400, height: 300 },
        collapsed: false,
      },
      {
        kind: "content",
        structureNodeId: "content",
        transform: { x: 10, y: 10, rotation: 0, scale: 1 },
        size: { width: 100, height: 80 },
        sizingMode: "frame",
      },
    ],
  };
}

describe("Space snapshot boundary", () => {
  it("parses a complete snapshot and rejects malformed elements", () => {
    const source = snapshot();
    expect(parseSpaceEditSnapshot(source)).toEqual(source);
    expect(() =>
      parseSpaceEditSnapshot({
        ...source,
        structure: [{ ...source.structure[0], locked: "false" }, source.structure[1]],
      }),
    ).toThrow(/expected boolean/u);
    expect(() =>
      parseSpaceEditSnapshot({
        ...source,
        layouts: [
          source.layouts[0],
          { ...source.layouts[1], transform: { x: 0, y: 0, rotation: 0, scale: 0 } },
        ],
      }),
    ).toThrow(/Too small/u);
  });

  it("rejects duplicate identities, invalid hierarchy and orphan layouts", () => {
    const source = snapshot();
    expect(() =>
      parseSpaceEditSnapshot({ ...source, structure: [source.structure[0], source.structure[0]] }),
    ).toThrow(/Duplicate structure node/u);
    expect(() =>
      parseSpaceEditSnapshot({
        ...source,
        structure: [source.structure[0], { ...source.structure[1], parentId: "missing" }],
      }),
    ).toThrow(/Invalid structure parent/u);
    expect(() =>
      parseSpaceEditSnapshot({
        ...source,
        layouts: [source.layouts[0], { ...source.layouts[1], structureNodeId: "missing" }],
      }),
    ).toThrow(/Orphan structure layout/u);
  });

  it("retains canonical sibling order and composed world geometry checks", () => {
    const source = snapshot();
    expect(() =>
      parseSpaceEditSnapshot({
        ...source,
        structure: [source.structure[0], { ...source.structure[1], order: 2 }],
      }),
    ).toThrow(/Noncanonical sibling order/u);
    expect(() =>
      parseSpaceEditSnapshot({
        ...source,
        layouts: [
          { ...source.layouts[0], position: { x: MAX_WORLD_COORDINATE, y: 0 } },
          { ...source.layouts[1], transform: { x: MAX_WORLD_COORDINATE, y: 0, rotation: 0, scale: 1 } },
        ],
      }),
    ).toThrow(/Invalid or degenerate canvas transform/u);
  });
});
