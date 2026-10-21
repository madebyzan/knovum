import { isSymmetricRelationKind } from "../../../model/relations/relation";
import type { CanvasSceneRelation } from "../scene/canvas-scene";

export function canvasRelationArrowheads(relation: CanvasSceneRelation) {
  return isSymmetricRelationKind(relation.label)
    ? { source: false, target: false }
    : { source: false, target: true };
}

export function connectorLabelPosition(relation: CanvasSceneRelation) {
  return relation.midpoint;
}
