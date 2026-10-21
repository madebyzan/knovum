import { describe, expect, it } from "vitest";

import { derivedOutputCounts, lineageRelationIds } from "../model/relations/lineage";
import type { SpaceRelation } from "../model/relations/relation";

function relation(
  id: string,
  sourceId: string,
  targetId: string,
  kind: SpaceRelation["kind"],
): SpaceRelation {
  return {
    id,
    spaceId: "space",
    sourceId,
    targetId,
    kind,
    rationale: "",
    status: "active",
    actorKind: "system",
    createdAt: 0,
    updatedAt: 0,
  };
}

// photo ← cutout ← subtitled, photo ← variant; the note only discusses the cutout.
const RELATIONS = [
  relation("cutout-of-photo", "cutout", "photo", "derives-from"),
  relation("subtitled-of-cutout", "subtitled", "cutout", "derives-from"),
  relation("variant-of-photo", "variant", "photo", "derives-from"),
  relation("note-on-cutout", "note", "cutout", "supports"),
];

describe("relation lineage", () => {
  it("explains a node by its whole history up to the origin and everything derived from it", () => {
    expect(lineageRelationIds(RELATIONS, ["cutout"])).toEqual(
      new Set(["cutout-of-photo", "subtitled-of-cutout"]),
    );
  });

  it("leaves out siblings, which share a source rather than a history", () => {
    expect(lineageRelationIds(RELATIONS, ["subtitled"])).toEqual(
      new Set(["subtitled-of-cutout", "cutout-of-photo"]),
    );
  });

  it("explains an origin by every output derived from it, directly or not", () => {
    expect(lineageRelationIds(RELATIONS, ["photo"])).toEqual(
      new Set(["cutout-of-photo", "subtitled-of-cutout", "variant-of-photo"]),
    );
  });

  it("explains nothing for content outside any derivation", () => {
    expect(lineageRelationIds(RELATIONS, ["note"])).toEqual(new Set());
  });

  it("terminates on a cycle", () => {
    const cycle = [
      relation("a-of-b", "a", "b", "derives-from"),
      relation("b-of-a", "b", "a", "derives-from"),
    ];

    expect(lineageRelationIds(cycle, ["a"])).toEqual(new Set(["a-of-b", "b-of-a"]));
  });

  it("counts the distinct outputs derived directly from each node", () => {
    expect(derivedOutputCounts(RELATIONS)).toEqual(
      new Map([
        ["photo", 2],
        ["cutout", 1],
      ]),
    );
  });
});
