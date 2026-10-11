import { describe, expect, it } from "vitest";

import { findSemanticRelationPath } from "../knowledge/model/relation-graph";
import { canonicalRelationEndpoints, type SpaceRelation } from "../model/relations/relation";
import { compareUtf8Binary } from "../model/text";

function relation(
  relationId: string,
  sourceId: string,
  targetId: string,
  kind: SpaceRelation["kind"],
): SpaceRelation {
  return {
    id: relationId,
    spaceId: "space",
    sourceId,
    targetId,
    kind,
    rationale: "",
    status: "active",
    actorKind: "human",
    createdAt: 1,
    updatedAt: 1,
  };
}

describe("Space semantic relation paths", () => {
  it("respects direction for supports", () => {
    const relations = [relation("r", "a", "b", "supports")];
    expect(findSemanticRelationPath(relations, "a", "b")).toEqual(["a", "b"]);
    expect(findSemanticRelationPath(relations, "b", "a")).toEqual([]);
  });

  it("canonicalizes symmetric endpoints with UTF-8 binary ordering", () => {
    const supplementary = "\u{10000}";
    const privateUse = "\uE000";
    // JavaScript UTF-16 ordering differs for this pair; Rust String and SQLite BINARY use UTF-8 bytes.
    expect(supplementary < privateUse).toBe(true);
    expect(compareUtf8Binary(supplementary, privateUse)).toBeGreaterThan(0);
    expect(canonicalRelationEndpoints(supplementary, privateUse, "contradicts")).toEqual([
      privateUse,
      supplementary,
    ]);
  });

  it("allows symmetric traversal for related and contradicts", () => {
    expect(findSemanticRelationPath([relation("r", "a", "b", "related")], "b", "a")).toEqual(["b", "a"]);
    expect(findSemanticRelationPath([relation("c", "a", "b", "contradicts")], "b", "a")).toEqual(["b", "a"]);
  });
});
