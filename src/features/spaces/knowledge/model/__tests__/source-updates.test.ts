import { describe, expect, it } from "vitest";

import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import { sourceUpdateCounts, type SpaceKnowledgeHealthIssue } from "../health";
import { currentSourceRevision, sourceHasRevision } from "../provenance";

const file = { sourceRevision: "bytes-1", knowledgeRevision: "node1:a" };

function stale(nodeId: string, sourceNodeId: string, sourceRevision: string): SpaceKnowledgeHealthIssue {
  return {
    id: `${nodeId}:${sourceNodeId}`,
    kind: "stale-provenance",
    severity: "warning",
    nodeId,
    title: "",
    sourceNodeId,
    sourceRevision,
  };
}

describe("whether a source changed", () => {
  it("is current while the source has the revision on either axis", () => {
    expect(sourceHasRevision(file, "bytes-1")).toBe(true);
    expect(sourceHasRevision(file, "node1:a")).toBe(true);
    expect(sourceHasRevision(file, "bytes-0")).toBe(false);
    expect(sourceHasRevision({}, "anything")).toBe(true);
  });

  it("re-takes at the bytes where there are bytes, else the content", () => {
    expect(currentSourceRevision(file)).toBe("bytes-1");
    expect(currentSourceRevision({ knowledgeRevision: "node1:b" })).toBe("node1:b");
  });
});

describe("source updates per item", () => {
  it("counts the sources still changed now, not those the last check saw", () => {
    const sources = new Map<string, SpaceNodePayload>([
      ["changed", { ...file, kind: "note" } as SpaceNodePayload],
      ["caught-up", { kind: "note", knowledgeRevision: "node1:new" } as SpaceNodePayload],
    ]);
    const issues = [
      stale("note", "changed", "bytes-0"),
      stale("note", "caught-up", "node1:new"),
      stale("other", "deleted", "x"),
      { ...stale("claim", "changed", "bytes-0"), kind: "unsupported-claim" as const },
    ];
    expect(sourceUpdateCounts(issues, sources)).toEqual(new Map([["note", 1]]));
  });
});
