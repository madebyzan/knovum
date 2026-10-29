import { describe, expect, it } from "vitest";

import type { KnowledgeChange } from "../../../contracts/generated/KnowledgeChange";
import type { ProposedChange } from "../../../contracts/generated/ProposedChange";
import { allChanges, toggleChange } from "../proposal-selection";

const change = { op: "create-object" } as KnowledgeChange;
// 0: claim A, 1: supports A, 2: provenance of A, 3: claim B, 4: relation A -> B
const changes: ProposedChange[] = [
  { change, requires: [] },
  { change, requires: [0] },
  { change, requires: [0] },
  { change, requires: [] },
  { change, requires: [0, 3] },
];

describe("proposal selection", () => {
  it("leaving out a claim leaves out everything that needs it", () => {
    expect([...toggleChange(changes, allChanges(changes), 0, false)].toSorted()).toEqual([3]);
  });

  it("choosing a change chooses what it needs", () => {
    expect([...toggleChange(changes, new Set(), 4, true)].toSorted()).toEqual([0, 3, 4]);
  });

  it("leaving out a leaf keeps the rest", () => {
    expect([...toggleChange(changes, allChanges(changes), 2, false)].toSorted()).toEqual([0, 1, 3, 4]);
  });
});
