import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../../model/relations/relation";
import type { CanvasDisplayNode } from "../../contract/canvas-display-node";
import type { CanvasNodeLayoutV1, CanvasRelationColor } from "../../contract/canvas-document";
import type { CanvasNodeInteraction } from "./canvas-node-interaction";
export interface CanvasSceneNode {
  node: CanvasDisplayNode;
  layout: CanvasNodeLayoutV1;
  interaction: CanvasNodeInteraction;
  paintOrder: number;
}
export interface CanvasSceneContentNode extends CanvasSceneNode {
  node: SpaceCanvasNode;
}
export interface CanvasSceneSection extends CanvasSceneNode {
  members: readonly CanvasSceneContentNode[];
}
export interface CanvasSceneRelation {
  id: string;
  key: string;
  kind: "relation";
  label: string;
  color: CanvasRelationColor;
  relation: SpaceRelation;
  sourceNodeId: string;
  targetNodeId: string;
  parallelOffset: number;
  points: number[];
  midpoint: { x: number; y: number };
  bounds: { x: number; y: number; width: number; height: number };
}
export interface CanvasScene {
  nodes: readonly CanvasSceneNode[];
  nodesById: ReadonlyMap<string, CanvasSceneNode>;
  contentNodes: readonly CanvasSceneContentNode[];
  sections: readonly CanvasSceneSection[];
  groups: readonly CanvasSceneNode[];
  relations: readonly CanvasSceneRelation[];
}
