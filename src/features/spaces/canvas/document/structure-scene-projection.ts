import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/edit-snapshot";
import type {
  StructureNodeId,
  StructureLayout,
  LocalTransform,
  StructureNode,
} from "../../model/structure/structure-contract";
import { validateStructureSnapshot } from "../../model/structure/structure-validation";
import { contentWorldBounds, mergeWorldBounds, type WorldBounds } from "../../model/structure/world-bounds";
import { composeTransforms, localTransformOf } from "../../model/structure/world-transform";
import type { ProjectedContent, ProjectedGroup, ProjectedSection, StructureScene } from "./structure-scene";
import { projectStructureRelations } from "./structure-scene-relations";

type ContentStructureNode = Extract<StructureNode, { kind: "content" }>;
type ContentLayout = Extract<StructureLayout, { kind: "content" }>;
type GroupStructureNode = Extract<StructureNode, { kind: "group" }>;
type SectionStructureNode = Extract<StructureNode, { kind: "section" }>;

/** One structure traversal owns content order, world geometry, folding, and relation representatives. */
export function projectStructureScene(
  snapshot: SpaceEditSnapshot,
  nodes: readonly SpaceCanvasNode[],
  relations: readonly SpaceRelation[],
  temporarilyExpandedSectionIds: ReadonlySet<StructureNodeId> = new Set(),
): StructureScene {
  const index = validateStructureSnapshot(snapshot.structure, snapshot.layouts);
  const layouts = new Map(snapshot.layouts.map((layout) => [layout.structureNodeId, layout]));
  const contentById = new Map(nodes.map((node) => [node.id, node]));
  const content: ProjectedContent[] = [];
  const groups: ProjectedGroup[] = [];
  const sections: ProjectedSection[] = [];
  const identity: LocalTransform = { x: 0, y: 0, rotation: 0, scale: 1 };
  const state: ProjectionState = {
    content,
    contentById,
    groups,
    index,
    layouts,
    sections,
    temporarilyExpandedSectionIds,
  };

  for (const root of index.childrenByParent.get(null) ?? [])
    walkStructure(state, root, {
      parentWorld: identity,
      sectionId: null,
      hidden: false,
      inheritedLocked: false,
    });
  sections.sort((left, right) => left.section.order - right.section.order);
  const projectedRelations = projectStructureRelations(
    relations,
    index,
    layouts,
    temporarilyExpandedSectionIds,
  );
  return { content, groups, sections, relations: projectedRelations };
}

interface ProjectionState {
  content: ProjectedContent[];
  contentById: ReadonlyMap<string, SpaceCanvasNode>;
  groups: ProjectedGroup[];
  index: ReturnType<typeof validateStructureSnapshot>;
  layouts: ReadonlyMap<StructureNodeId, StructureLayout>;
  sections: ProjectedSection[];
  temporarilyExpandedSectionIds: ReadonlySet<StructureNodeId>;
}

interface WalkState {
  parentWorld: LocalTransform;
  sectionId: StructureNodeId | null;
  hidden: boolean;
  inheritedLocked: boolean;
}

function walkStructure(
  state: ProjectionState,
  structureNode: StructureNode,
  traversal: WalkState,
): { bounds: WorldBounds | undefined; contentCount: number } {
  const layout = state.layouts.get(structureNode.id);
  if (!layout || structureNode.kind !== layout.kind)
    throw new Error(`Missing layout for ${structureNode.id}`);
  const world = composeTransforms(traversal.parentWorld, localTransformOf(layout));
  const locked = traversal.inheritedLocked || structureNode.locked;
  if (structureNode.kind === "content") {
    if (layout.kind !== "content") throw new Error(`Layout kind mismatch for ${structureNode.id}`);
    return projectContent(state, { structureNode, layout, world, traversal });
  }
  if (layout.kind === "content") throw new Error(`Layout kind mismatch for ${structureNode.id}`);
  return projectContainer(state, { structureNode, layout, world, traversal, locked });
}

function projectContent(
  state: ProjectionState,
  projection: {
    structureNode: ContentStructureNode;
    layout: ContentLayout;
    world: LocalTransform;
    traversal: WalkState;
  },
) {
  const { structureNode, layout, world, traversal } = projection;
  const node = state.contentById.get(structureNode.nodeId);
  if (!node) throw new Error(`Active content missing for ${structureNode.nodeId}`);
  const bounds = contentWorldBounds(layout, world);
  state.content.push({
    structureNode,
    layout,
    node,
    world,
    bounds,
    sectionId: traversal.sectionId,
    paintOrder: state.content.length,
    visible: !traversal.hidden && !node.hidden,
  });
  return { bounds, contentCount: 1 };
}

function projectContainer(
  state: ProjectionState,
  projection: {
    structureNode: GroupStructureNode | SectionStructureNode;
    layout: Exclude<StructureLayout, ContentLayout>;
    world: LocalTransform;
    traversal: WalkState;
    locked: boolean;
  },
) {
  const { structureNode, layout, world, traversal, locked } = projection;
  const sectionId = structureNode.kind === "section" ? structureNode.id : traversal.sectionId;
  const temporarilyExpanded =
    structureNode.kind === "section" && state.temporarilyExpandedSectionIds.has(structureNode.id);
  const hidden = traversal.hidden || (layout.kind === "section" && layout.collapsed && !temporarilyExpanded);
  let bounds: WorldBounds | undefined;
  let contentCount = 0;
  for (const child of state.index.childrenByParent.get(structureNode.id) ?? []) {
    const childProjection = walkStructure(state, child, {
      parentWorld: world,
      sectionId,
      hidden,
      inheritedLocked: locked,
    });
    bounds = mergeWorldBounds(bounds, childProjection.bounds);
    contentCount += childProjection.contentCount;
  }
  if (structureNode.kind === "group") recordGroup({ state, group: structureNode, traversal, locked, bounds });
  else recordSection({ state, section: structureNode, layout, contentCount, locked, temporarilyExpanded });
  return { bounds, contentCount };
}

function recordGroup(input: {
  state: ProjectionState;
  group: GroupStructureNode;
  traversal: WalkState;
  locked: boolean;
  bounds: WorldBounds | undefined;
}) {
  const { state, group, traversal, locked, bounds } = input;
  if (!bounds) throw new Error(`Empty Group: ${group.id}`);
  state.groups.push({ group, bounds, sectionId: traversal.sectionId, visible: !traversal.hidden, locked });
}

function recordSection(input: {
  state: ProjectionState;
  section: SectionStructureNode;
  layout: Exclude<StructureLayout, ContentLayout>;
  contentCount: number;
  locked: boolean;
  temporarilyExpanded: boolean;
}) {
  const { state, section, layout, contentCount, locked, temporarilyExpanded } = input;
  if (layout.kind !== "section") throw new Error(`Missing Section layout for ${section.id}`);
  const { x, y } = layout.position;
  state.sections.push({
    section,
    layout,
    bounds: { left: x, top: y, right: x + layout.size.width, bottom: y + layout.size.height },
    contentCount,
    locked,
    temporarilyExpanded,
  });
}
