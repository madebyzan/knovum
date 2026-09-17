import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../../model/relations/relation";
import type { CanvasNodeLayoutV1, CanvasRelationColor } from "../../contract/canvas-document";
import type { CanvasNodeInteraction } from "./canvas-node-interaction";
import type { CanvasRelationPath } from "./relation-path";

export interface CanvasSceneNode {
  interaction: CanvasNodeInteraction;
  layout: CanvasNodeLayoutV1;
  node: SpaceCanvasNode;
  paintOrder: number;
}

export interface CanvasSceneSection extends CanvasSceneNode {
  members: readonly CanvasSceneNode[];
}

export interface CanvasSceneRelation {
  bounds: { x: number; y: number; width: number; height: number };
  id: string;
  key: string;
  kind: "relation";
  label: SpaceRelation["kind"];
  color: CanvasRelationColor;
  midpoint: { x: number; y: number };
  points: CanvasRelationPath;
  relation: SpaceRelation;
  sourceNodeId: string;
  targetNodeId: string;
  parallelOffset: number;
}

export interface CanvasScene {
  contentNodes: readonly CanvasSceneNode[];
  nodes: readonly CanvasSceneNode[];
  nodesById: ReadonlyMap<string, CanvasSceneNode>;
  sections: readonly CanvasSceneSection[];
  relations: readonly CanvasSceneRelation[];
}
