import type { SpaceRelation } from "../../../model/relations/relation";

export function canvasAutoLayoutEdges(relations: readonly SpaceRelation[]) {
  return relations.flatMap(({ id, sourceId, status, targetId }) =>
    status === "invalidated" ? [] : [{ id, sourceId, targetId }],
  );
}
