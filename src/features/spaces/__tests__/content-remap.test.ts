import { describe, expect, it } from "vitest";

import { remapGroundedCitation } from "../package/content-remap";

const snapshot = {
  index: 1,
  state: "current" as const,
  chunkId: "node:old-node",
  nodeId: "old-node",
  nodeTitle: "Object",
  sourceRevision: "node1:old",
  locator: { sourceRevision: "node1:old", location: { kind: "node" as const } },
  quote: "Object metadata",
};

describe("Space package content remapping", () => {
  it("rebinds canonical citations to the remapped node revision", () => {
    const remapped = remapGroundedCitation(
      snapshot,
      { nodes: new Map([["old-node", "new-node"]]), chunks: new Map() },
      new Map([["new-node", "node1:new"]]),
    );
    expect(remapped).toMatchObject({
      state: "current",
      nodeId: "new-node",
      chunkId: "node:new-node",
      sourceRevision: "node1:new",
      locator: { sourceRevision: "node1:new" },
    });
  });

  it("keeps detached historical snapshots immutable", () => {
    const detached = { ...snapshot, state: "detached" as const, chunkId: undefined, nodeId: undefined };
    const remapped = remapGroundedCitation(detached, { nodes: new Map(), chunks: new Map() }, new Map());
    expect(remapped).toEqual(detached);
  });
});
