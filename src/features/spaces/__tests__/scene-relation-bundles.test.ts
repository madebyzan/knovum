import { describe, expect, it } from "vitest";

import { projectSceneRelations, type PlacedRelation } from "../canvas/konva/scene/scene-relations";
import type { SpaceRelation } from "../model/relations/relation";

function placed(
  id: string,
  kind: SpaceRelation["kind"],
  sourceStructureNodeId: string,
  targetStructureNodeId: string,
): PlacedRelation {
  return {
    relation: {
      id,
      spaceId: "space",
      sourceId: `${id}-source`,
      targetId: `${id}-target`,
      kind,
      rationale: "",
      status: "active",
      actorKind: "human",
      createdAt: 0,
      updatedAt: 0,
    },
    sourceStructureNodeId,
    targetStructureNodeId,
  };
}

const ENDPOINTS = new Map(
  ["section", "source", "other"].map((id, index) => [
    id,
    { frame: { x: index * 400, y: 0, width: 200, height: 100, rotation: 0 }, previewId: id },
  ]),
);

// Three outputs inside a collapsed section all derive from one source; two of the outputs also
// support it, and one supports another node.
const PLACED = [
  placed("a", "derives-from", "section", "source"),
  placed("b", "derives-from", "section", "source"),
  placed("c", "derives-from", "section", "source"),
  placed("d", "supports", "section", "source"),
  placed("e", "supports", "section", "source"),
  placed("f", "supports", "section", "other"),
];

describe("scene relations", () => {
  it("draws relations of one kind between the same visible endpoints as one line", () => {
    const lines = projectSceneRelations(PLACED, ENDPOINTS, new Map());

    expect(lines.map(({ label, relations }) => [label, relations.map(({ id }) => id)])).toEqual([
      ["derives-from", ["a", "b", "c"]],
      ["supports", ["d", "e"]],
      ["supports", ["f"]],
    ]);
    expect(lines.map(({ lineage }) => lineage)).toEqual([true, false, false]);
  });

  it("keeps a single relation's identity and spaces the lines that share endpoints", () => {
    const [lineage, support, single] = projectSceneRelations(PLACED, ENDPOINTS, new Map());

    expect(single).toMatchObject({ id: "f", key: "relation:f" });
    expect(single!.parallelOffset).toBeCloseTo(0);
    expect(lineage!.id).not.toBe(support!.id);
    expect(Math.abs(lineage!.parallelOffset - support!.parallelOffset)).toBe(20);
  });

  it("colours a line only when its relations agree on a colour", () => {
    const colors = new Map([
      ["a", "red" as const],
      ["b", "red" as const],
      ["c", "red" as const],
      ["d", "red" as const],
      ["e", "blue" as const],
    ]);

    const lines = projectSceneRelations(PLACED, ENDPOINTS, colors);

    expect(lines.map(({ color }) => color)).toEqual(["red", "neutral", "neutral"]);
  });

  it("joins symmetric relations regardless of the direction they were drawn in", () => {
    const lines = projectSceneRelations(
      [placed("x", "related", "section", "source"), placed("y", "related", "source", "section")],
      ENDPOINTS,
      new Map(),
    );

    expect(lines.map(({ relations }) => relations.map(({ id }) => id))).toEqual([["x", "y"]]);
  });
});
