import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodeKind } from "../../model/nodes/node-payload";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { SECTION_HEADER_HEIGHT } from "../../model/structure/section-metrics";
import { mergeWorldBounds, type WorldBounds } from "../../model/structure/world-bounds";
import type { CanvasPoint } from "../contracts/canvas-editor-port";
import type { ProjectedContent, ProjectedRelation, ProjectedSection } from "../document/structure-scene";
import { projectStructureScene } from "../document/structure-scene-projection";
import type { CanvasNodeAppearance, CanvasRelationColor } from "../schema/document";

export interface MinimapNode {
  readonly id: string;
  readonly kind: SpaceNodeKind;
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
  readonly rotation: number;
  readonly fillPreset?: CanvasNodeAppearance["fill"] | undefined;
  readonly customFill?: string | undefined;
  readonly selected: boolean;
  readonly focused: boolean;
}

export interface MinimapSection {
  readonly id: string;
  readonly bounds: WorldBounds;
  readonly collapsed: boolean;
  readonly fillPreset?: CanvasNodeAppearance["fill"] | undefined;
  readonly customFill?: string | undefined;
  readonly selected: boolean;
}

export interface MinimapRelation {
  readonly id: string;
  readonly from: CanvasPoint;
  readonly to: CanvasPoint;
  readonly color: CanvasRelationColor;
}

export interface MinimapScene {
  readonly sections: readonly MinimapSection[];
  readonly nodes: readonly MinimapNode[];
  readonly relations: readonly MinimapRelation[];
  readonly contentBounds: WorldBounds | undefined;
  readonly hasFocus: boolean;
}

export interface ProjectMinimapSceneInput {
  readonly snapshot: SpaceEditSnapshot;
  readonly nodes: readonly SpaceCanvasNode[];
  readonly relations: readonly SpaceRelation[];
  readonly selectedNodeIds: ReadonlySet<string>;
  readonly selectedStructureNodeIds: ReadonlySet<string>;
  readonly focusNodeIds: ReadonlySet<string>;
}

/** Projects the active Space structure and selection state into lightweight minimap primitives. */
export function projectMinimapScene(input: ProjectMinimapSceneInput): MinimapScene {
  const structure = projectStructureScene({
    snapshot: input.snapshot,
    nodes: input.nodes,
    relations: input.relations,
  });
  const endpointByStructureId = new Map<string, CanvasPoint>();
  const projectedSections = collectSections(
    structure.sections,
    input.selectedStructureNodeIds,
    endpointByStructureId,
  );
  const projectedNodes = collectNodes(
    structure.content,
    input.selectedNodeIds,
    input.focusNodeIds,
    endpointByStructureId,
  );
  const colorByRelationId = new Map(
    input.snapshot.relationStyles.map((style) => [style.relationId, style.color]),
  );
  return {
    sections: projectedSections.items,
    nodes: projectedNodes.items,
    relations: collectRelations(structure.relations, endpointByStructureId, colorByRelationId),
    contentBounds: mergeWorldBounds(projectedSections.bounds, projectedNodes.bounds),
    hasFocus: input.focusNodeIds.size > 0,
  };
}

function collectSections(
  projected: readonly ProjectedSection[],
  selectedIds: ReadonlySet<string>,
  endpointByStructureId: Map<string, CanvasPoint>,
) {
  let bounds: WorldBounds | undefined;
  const items: MinimapSection[] = [];
  for (const item of projected) {
    const collapsed = item.layout.collapsed && !item.temporarilyExpanded;
    const itemBounds = collapsed
      ? { ...item.bounds, bottom: item.bounds.top + SECTION_HEADER_HEIGHT }
      : item.bounds;
    bounds = mergeWorldBounds(bounds, itemBounds);
    endpointByStructureId.set(item.section.id, boundsCenter(itemBounds));
    items.push({
      id: item.section.id,
      bounds: itemBounds,
      collapsed,
      fillPreset: item.layout.appearance?.fill,
      customFill: item.layout.appearance?.customFill,
      selected: selectedIds.has(item.section.id),
    });
  }
  return { items, bounds };
}

function collectNodes(
  projected: readonly ProjectedContent[],
  selectedIds: ReadonlySet<string>,
  focusIds: ReadonlySet<string>,
  endpointByStructureId: Map<string, CanvasPoint>,
) {
  let bounds: WorldBounds | undefined;
  const items: MinimapNode[] = [];
  for (const item of projected) {
    if (!item.visible) continue;
    bounds = mergeWorldBounds(bounds, item.bounds);
    endpointByStructureId.set(item.structureNode.id, boundsCenter(item.bounds));
    items.push({
      id: item.node.id,
      kind: item.node.data.kind,
      x: item.world.x,
      y: item.world.y,
      width: item.layout.size.width * item.world.scale,
      height: item.layout.size.height * item.world.scale,
      rotation: item.world.rotation,
      fillPreset: item.layout.appearance?.fill,
      customFill: item.layout.appearance?.customFill,
      selected: selectedIds.has(item.node.id),
      focused: focusIds.has(item.node.id),
    });
  }
  return { items, bounds };
}

function collectRelations(
  projected: readonly ProjectedRelation[],
  endpointByStructureId: ReadonlyMap<string, CanvasPoint>,
  colorByRelationId: ReadonlyMap<string, CanvasRelationColor>,
): MinimapRelation[] {
  const result: MinimapRelation[] = [];
  for (const item of projected) {
    const from = endpointByStructureId.get(item.sourceStructureNodeId);
    const to = endpointByStructureId.get(item.targetStructureNodeId);
    if (!from || !to) continue;
    result.push({
      id: item.relation.id,
      from,
      to,
      color: colorByRelationId.get(item.relation.id) ?? "neutral",
    });
  }
  return result;
}

export function boundsCenter(bounds: WorldBounds): CanvasPoint {
  return {
    x: (bounds.left + bounds.right) / 2,
    y: (bounds.top + bounds.bottom) / 2,
  };
}
