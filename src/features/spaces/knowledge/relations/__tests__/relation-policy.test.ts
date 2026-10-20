import { expect, it } from "vitest";

import type { SpaceRelation } from "../../../model/relations/relation";
import { reviseRelation } from "../relation-policy";

function relation(relationId: string, rationale: string): SpaceRelation {
  return {
    id: relationId,
    spaceId: "space",
    sourceId: "left",
    targetId: "right",
    kind: "related",
    rationale,
    status: "active",
    actorKind: "human",
    createdAt: 1,
    updatedAt: 1,
  };
}

it.each([
  [{ kind: "related" as const }, "original"],
  [{ rationale: "revised" }, "revised"],
])("preserves the edited relation when its kind does not change", (patch, rationale) => {
  const current = relation("current", "original");
  const decision = reviseRelation(current, patch, [current, relation("existing", "other")]);
  expect(decision.kind).toBe("updated");
  expect(decision.relation.id).toBe("current");
  expect(decision.relation.rationale).toBe(rationale);
});
