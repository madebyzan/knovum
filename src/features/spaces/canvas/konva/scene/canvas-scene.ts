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

/**
 * One drawn line. Relations of one kind between the same visible endpoints share a line — a
 * collapsed section stands in for everything inside it, so its relations to one node would
 * otherwise fan out as parallel copies — and `relations` holds every relation the line stands for.
 */
export interface CanvasSceneRelation {
  bounds: { x: number; y: number; width: number; height: number };
  id: string;
  key: string;
  kind: "relation";
  label: SpaceRelation["kind"];
  /** Drawn only when it explains the selection; see `isLineageRelation`. */
  lineage: boolean;
  color: CanvasRelationColor;
  midpoint: { x: number; y: number };
  points: CanvasRelationPath;
  relations: readonly SpaceRelation[];
  sourceNodeId: string;
  sourceFrame: CanvasNodeLayoutV1["frame"];
  targetNodeId: string;
  targetFrame: CanvasNodeLayoutV1["frame"];
  parallelOffset: number;
}

interface CanvasSceneSectionBackdrop {
  appearance: Extract<StructureLayout, { kind: "section" }>["appearance"];
  bounds: { x: number; y: number; width: number; height: number };
  id: string;
}

export interface CanvasScene {
  nodes: readonly CanvasSceneNode[];
  nodesById: ReadonlyMap<string, CanvasSceneNode>;
  sectionBackdrops: readonly CanvasSceneSectionBackdrop[];
  relations: readonly CanvasSceneRelation[];
  /** The line each drawn relation is part of, by relation id. */
  relationKeyByRelationId: ReadonlyMap<string, string>;
}
