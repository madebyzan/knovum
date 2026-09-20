import { describe, expect, it } from "vitest";

import { resolveStructureKnowledgeScope } from "../knowledge/assistant/structure-scope";
import type { SpaceStructure } from "../model/structure/structure-schema";

const structure: SpaceStructure = {
  version: 1,
  items: [
    { id: "section", kind: "section", title: "Sources", parentId: null, order: 0, locked: false },
    { id: "group", kind: "group", title: "Evidence", parentId: "section", order: 0, locked: false },
    { id: "nested", kind: "group", title: "Details", parentId: "group", order: 0, locked: false },
    { id: "a", nodeId: "a", kind: "content", parentId: "nested", order: 0, locked: false },
    { id: "b", nodeId: "b", kind: "content", parentId: "group", order: 1, locked: true },
    { id: "c", nodeId: "c", kind: "content", parentId: "section", order: 1, locked: false },
    { id: "outside", nodeId: "outside", kind: "content", parentId: null, order: 1, locked: false },
    { id: "other", kind: "section", title: "Other", parentId: null, order: 2, locked: false },
    { id: "d", nodeId: "d", kind: "content", parentId: "other", order: 0, locked: false },
  ],
};
const eligible = new Set(["a", "b", "c", "outside", "d"]);

function scope(ids: readonly string[], allowed = eligible) {
  return resolveStructureKnowledgeScope(structure, ids, allowed);
}

describe("structure-backed knowledge scopes", () => {
  it("expands a selected nested Group without counting containers as content", () => {
    expect(scope(["group"])).toEqual({
      selection: ["a", "b"], section: ["a", "b", "c"], sectionId: "section",
    });
  });

  it("deduplicates selected ancestors and descendants", () => {
    expect(scope(["group", "nested", "a", "group"]).selection).toEqual(["a", "b"]);
  });

  it("includes every Section descendant independently of rendering and locking", () => {
    expect(scope(["section"]).selection).toEqual(["a", "b", "c"]);
  });

  it("applies semantic eligibility after traversing the complete structure", () => {
    expect(scope(["section"], new Set(["a", "c"]))).toEqual({
      selection: ["a", "c"], section: ["a", "c"], sectionId: "section",
    });
  });

  it("does not select an arbitrary Section for a cross-Section selection", () => {
    expect(scope(["a", "d"])).toEqual({ selection: ["a", "d"], section: [], sectionId: undefined });
  });

  it("does not implicitly exclude selected root content from the scope", () => {
    expect(scope(["a", "outside"])).toEqual({
      selection: ["a", "outside"], section: [], sectionId: undefined,
    });
  });

  it("drops stale transient selection identities without modifying the document", () => {
    const before = structuredClone(structure);
    expect(scope(["removed", "a"]).selection).toEqual(["a"]);
    expect(structure).toEqual(before);
  });

  it("does not invent a Section for an empty selection", () => {
    expect(scope([])).toEqual({ selection: [], section: [], sectionId: undefined });
  });
});
