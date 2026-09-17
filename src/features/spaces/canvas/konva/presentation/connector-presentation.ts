import { isSymmetricRelationKind } from "../../../model/relations/relation";
import type { CanvasSceneRelation } from "../scene/scene-projection";

export function canvasRelationArrowheads(relation: CanvasSceneRelation) {
  return isSymmetricRelationKind(relation.relation.kind)
    ? { source: false, target: false }
    : { source: false, target: true };
}

export function connectorLabelPosition(relation: CanvasSceneRelation) {
  return relation.midpoint;
}
