import { describe, expect, it, vi } from "vitest";

import { resolveImportedRelations } from "../json-canvas-relations";

describe("JSON Canvas relation mapping", () => {
  it("maps standard edges into the authoritative semantic relation model", () => {
    vi.spyOn(crypto, "randomUUID").mockReturnValue("00000000-0000-4000-8000-000000000001");

    expect(
      resolveImportedRelations(
        [{ id: "edge", fromNode: "a", toNode: "b", label: "supports" }],
        [
          { sourceId: "a", nodeId: "node-a" },
          { sourceId: "b", nodeId: "node-b" },
        ],
        "space",
      ),
    ).toEqual([
      expect.objectContaining({
        id: "00000000-0000-4000-8000-000000000001",
        spaceId: "space",
        sourceId: "node-a",
        targetId: "node-b",
        kind: "related",
        rationale: "supports",
      }),
    ]);
  });
});
