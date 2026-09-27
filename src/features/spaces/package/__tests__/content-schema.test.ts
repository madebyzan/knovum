import { describe, expect, it } from "vitest";

import { nodeSchema, spaceSchema } from "../content-schema";
import { stripSourceSpace } from "../restore-space";

const timestamps = { createdAt: 0, updatedAt: 0 };

describe("package content boundaries", () => {
  it("accepts current content and rejects obsolete content kinds", () => {
    expect(nodeSchema.parse({ id: "node", kind: "note", data: {}, ...timestamps }).id).toBe("node");
    expect(nodeSchema.safeParse({ id: "node", kind: "section", data: {}, ...timestamps }).success).toBe(
      false,
    );
  });

  it("rejects unknown fields and invalid package timestamps", () => {
    const space = {
      id: "space",
      kind: "space",
      title: "Title",
      archivedAt: null,
      lastOpenedAt: 0,
      ...timestamps,
    };
    expect(spaceSchema.parse(space).id).toBe("space");
    expect(spaceSchema.safeParse({ ...space, extra: true }).success).toBe(false);
    expect(spaceSchema.safeParse({ ...space, updatedAt: -1 }).success).toBe(false);
  });

  it("removes source ownership only after checking it", () => {
    const record = { id: "node", spaceId: "source", title: "Title" };
    expect(stripSourceSpace(record, "source")).toEqual({ id: "node", title: "Title" });
    expect(record.spaceId).toBe("source");
    expect(() => stripSourceSpace(record, "other")).toThrow("different source space");
  });
});
