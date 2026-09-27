import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../../model/relations/relation";
import type { StructureLayout } from "../../../model/structure/schema/structure";
import type { CanvasNodeInteraction } from "../../canvas-node-interaction";
import type { CanvasNodeLayoutV1, CanvasRelationColor } from "../../schema/document";
import type { CanvasRelationPath } from "./relation-path";

export interface CanvasSceneNode {
  interaction: CanvasNodeInteraction;
  layout: CanvasNodeLayoutV1;
  node: SpaceCanvasNode;
  paintOrder: number;
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
  sourceFrame: CanvasNodeLayoutV1["frame"];
  targetNodeId: string;
  targetFrame: CanvasNodeLayoutV1["frame"];
  parallelOffset: number;
}

export interface CanvasSceneSectionBackdrop {
  appearance: Extract<StructureLayout, { kind: "section" }>["appearance"];
  bounds: { x: number; y: number; width: number; height: number };
  id: string;
}

export interface CanvasScene {
  contentNodes: readonly CanvasSceneNode[];
  nodes: readonly CanvasSceneNode[];
  nodesById: ReadonlyMap<string, CanvasSceneNode>;
  sectionBackdrops: readonly CanvasSceneSectionBackdrop[];
  relations: readonly CanvasSceneRelation[];
}
