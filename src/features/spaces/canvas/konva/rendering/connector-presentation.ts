import { isSymmetricRelationKind } from "../../../model/relations/relation";
import type { CanvasSceneRelation } from "../scene/canvas-scene";

let measureContext: CanvasRenderingContext2D | null | undefined;

export function canvasRelationArrowheads(relation: CanvasSceneRelation) {
  return isSymmetricRelationKind(relation.label)
    ? { source: false, target: false }
    : { source: false, target: true };
}

export function connectorLabelPosition(relation: CanvasSceneRelation) {
  return relation.midpoint;
}

export function measureRelationLabelWidth(text: string, fontFamily: string, fontSize: number) {
  if (measureContext === undefined) {
    measureContext =
      typeof document === "undefined" ? null : document.createElement("canvas").getContext("2d");
  }
  if (!measureContext) return Math.max(24, text.length * fontSize);
  measureContext.font = `${fontSize}px ${fontFamily}`;
  return Math.ceil(measureContext.measureText(text).width);
}
