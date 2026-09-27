import { describe, expect, it } from "vitest";

import { clipboardRelationSchema } from "../clipboard/schema/relation";
import { relationSchema, toSpaceRelation } from "../model/relations/schema/relation";

const relation = {
  id: "r",
  spaceId: "s",
  sourceId: "a",
  targetId: "b",
  kind: "supports",
  rationale: "",
  status: "active",
  statusBeforeInvalidation: null,
  confidence: null,
  actorKind: "human",
  actorId: null,
  createdAt: 1,
  updatedAt: 1,
};

describe("relation boundary", () => {
  it("requires a prior state for invalidation and none for other states", () => {
    expect(relationSchema.safeParse({ ...relation, status: "invalidated" }).success).toBe(false);
    expect(
      relationSchema.safeParse({ ...relation, status: "invalidated", statusBeforeInvalidation: "active" })
        .success,
    ).toBe(true);
    expect(relationSchema.safeParse({ ...relation, statusBeforeInvalidation: "active" }).success).toBe(false);
  });

  it("normalizes clipboard optionals without weakening durable record validation", () => {
    const {
      confidence: _confidence,
      actorId: _actorId,
      statusBeforeInvalidation: _status,
      ...copied
    } = relation;
    expect(relationSchema.safeParse(copied).success).toBe(false);
    const normalized = clipboardRelationSchema.parse(copied);
    expect(normalized.statusBeforeInvalidation).toBeNull();
    expect(toSpaceRelation(normalized)).not.toHaveProperty("confidence");
    expect(toSpaceRelation(normalized)).not.toHaveProperty("actorId");
  });
});
