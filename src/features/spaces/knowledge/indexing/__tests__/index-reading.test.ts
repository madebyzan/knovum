import { describe, expect, it } from "vitest";

import type { KnowledgeIndexState } from "../../../contracts/generated/KnowledgeIndexState";
import { readKnowledgeIndex } from "../index-reading";

function state(overrides: Partial<KnowledgeIndexState> = {}): KnowledgeIndexState {
  return {
    total: 100,
    indexed: 100,
    pending: 0,
    searchable: true,
    rebuilding: false,
    indexing: false,
    indexedAt: 1_700_000_000_000,
    ...overrides,
  };
}

describe("retrieval index reading", () => {
  it("offers nothing to press when the index already covers the space", () => {
    const reading = readKnowledgeIndex(state(), false);

    // A Space that is already searchable has nothing to repair, and a button that re-encodes it
    // anyway is a second entry point that can only disagree with the one the panel reports.
    expect(reading).toMatchObject({ condition: "current", action: undefined, coverage: 100 });
  });

  it("separates a space that cannot answer from one that is merely behind", () => {
    expect(readKnowledgeIndex(state({ searchable: false, pending: 100 }), false)).toMatchObject({
      condition: "unbuilt",
      action: "build",
      coverage: 0,
    });
    expect(readKnowledgeIndex(state({ pending: 25 }), false)).toMatchObject({
      condition: "incomplete",
      action: "fill",
      coverage: 75,
      indexed: 75,
    });
  });

  it("asks for nothing while a pass holds the space, whoever started it", () => {
    // A pass this window started is live before the next reading arrives; one started by work that
    // needed the index is only ever visible in the reading.
    expect(readKnowledgeIndex(state({ pending: 40 }), true).action).toBeUndefined();
    expect(readKnowledgeIndex(state({ pending: 40, indexing: true }), false)).toMatchObject({
      condition: "indexing",
      action: undefined,
    });
  });

  it("reports a space with nothing to index as complete rather than broken", () => {
    expect(readKnowledgeIndex(state({ total: 0, pending: 0, searchable: false }), false)).toMatchObject({
      condition: "empty",
      action: undefined,
      coverage: 100,
    });
    expect(readKnowledgeIndex(undefined, false).condition).toBe("empty");
  });
});
