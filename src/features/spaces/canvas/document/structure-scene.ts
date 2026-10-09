import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type {
  LocalTransform,
  StructureLayout,
  StructureNode,
  StructureNodeId,
} from "../../model/structure/schema/structure";
import type { WorldBounds } from "../../model/structure/world-bounds";

export interface ProjectedContent {
  structureNode: Extract<StructureNode, { kind: "content" }>;
  layout: Extract<StructureLayout, { kind: "content" }>;
  node: SpaceCanvasNode;
  world: LocalTransform;
  bounds: WorldBounds;
  sectionId: StructureNodeId | null;
  paintOrder: number;
  visible: boolean;
}

export interface ProjectedGroup {
  group: Extract<StructureNode, { kind: "group" }>;
  bounds: WorldBounds;
  sectionId: StructureNodeId | null;
  visible: boolean;
  locked: boolean;
}

export interface ProjectedSection {
  section: Extract<StructureNode, { kind: "section" }>;
  layout: Extract<StructureLayout, { kind: "section" }>;
  bounds: WorldBounds;
  contentCount: number;
  locked: boolean;
  temporarilyExpanded: boolean;
}

export interface ProjectedRelation {
  relation: SpaceRelation;
  sourceStructureNodeId: StructureNodeId;
  targetStructureNodeId: StructureNodeId;
}

export interface StructureScene {
  content: readonly ProjectedContent[];
  groups: readonly ProjectedGroup[];
  sections: readonly ProjectedSection[];
  relations: readonly ProjectedRelation[];
}

export const SECTION_HEADER_HEIGHT = 40;
