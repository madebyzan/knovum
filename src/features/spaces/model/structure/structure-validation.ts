import type { StructureLayout, StructureNode } from "./structure-contract";
import {
  structureLayoutSchema,
  MAX_STRUCTURE_NODES,
  MAX_STRUCTURE_DEPTH,
  structureNodeSchema,
} from "./structure-contract";
import { createStructureIndex, type StructureIndex } from "./structure-index";
import { contentWorldBounds } from "./world-bounds";
import { composeTransforms, localTransformOf } from "./world-transform";

/** Validate a complete active Space snapshot before projecting or committing it. */
export function validateStructureSnapshot(
  structure: readonly StructureNode[],
  layouts: readonly StructureLayout[],
): StructureIndex {
  if (structure.length > MAX_STRUCTURE_NODES || layouts.length !== structure.length)
    throw new Error("Structure and layout structureNode counts do not match or exceed the limit");
  const structureNodeIds = validateStructureNodes(structure);
  const layoutById = validateLayouts(layouts, structureNodeIds);
  const index = createStructureIndex(structure);
  validateHierarchy(structure, layoutById, index);
  validateTraversal(structure, layoutById, index);
  return index;
}

function validateStructureNodes(structure: readonly StructureNode[]) {
  const structureNodeIds = new Set<string>();
  const nodeIds = new Set<string>();
  for (const structureNode of structure) {
    structureNodeSchema.parse(structureNode);
    if (structureNodeIds.has(structureNode.id))
      throw new Error(`Duplicate structure node: ${structureNode.id}`);
    structureNodeIds.add(structureNode.id);
    if (structureNode.kind !== "content") continue;
    if (nodeIds.has(structureNode.nodeId))
      throw new Error(`Duplicate content reference: ${structureNode.nodeId}`);
    nodeIds.add(structureNode.nodeId);
  }
  return structureNodeIds;
}

function validateLayouts(layouts: readonly StructureLayout[], structureNodeIds: ReadonlySet<string>) {
  const layoutById = new Map<string, StructureLayout>();
  for (const layout of layouts) {
    structureLayoutSchema.parse(layout);
    if (layoutById.has(layout.structureNodeId))
      throw new Error(`Duplicate structure layout: ${layout.structureNodeId}`);
    layoutById.set(layout.structureNodeId, layout);
    if (!structureNodeIds.has(layout.structureNodeId))
      throw new Error(`Orphan structure layout: ${layout.structureNodeId}`);
  }
  return layoutById;
}

function validateHierarchy(
  structure: readonly StructureNode[],
  layoutById: ReadonlyMap<string, StructureLayout>,
  index: StructureIndex,
) {
  for (const structureNode of structure) validateStructureRelationship(structureNode, layoutById, index);
  for (const [parentId, siblings] of index.childrenByParent) {
    for (const [order, structureNode] of siblings.entries())
      validateSiblingOrder(parentId, structureNode, order);
  }
}

function validateStructureRelationship(
  structureNode: StructureNode,
  layoutById: ReadonlyMap<string, StructureLayout>,
  index: StructureIndex,
) {
  if (layoutById.get(structureNode.id)?.kind !== structureNode.kind)
    throw new Error(`Structure layout missing or mismatched: ${structureNode.id}`);
  if (structureNode.kind === "section" && structureNode.parentId !== null)
    throw new Error(`Section must be a root: ${structureNode.id}`);
  if (structureNode.parentId === null) return;
  const parent = index.structureById.get(structureNode.parentId);
  if (!parent || parent.kind === "content") throw new Error(`Invalid structure parent: ${structureNode.id}`);
}

function validateSiblingOrder(parentId: string | null, structureNode: StructureNode, order: number) {
  if (structureNode.order !== order)
    throw new Error(`Noncanonical sibling order under: ${parentId ?? "root"}`);
}

function validateTraversal(
  structure: readonly StructureNode[],
  layoutById: ReadonlyMap<string, StructureLayout>,
  index: StructureIndex,
) {
  const visited = new Set<string>();
  const queue = (index.childrenByParent.get(null) ?? []).map((structureNode) => ({
    structureNode,
    depth: 1,
    world: { x: 0, y: 0, rotation: 0, scale: 1 },
  }));
  for (let cursor = 0; cursor < queue.length; cursor++) {
    const { structureNode, depth, world } = queue[cursor]!;
    validateTraversalPosition(structureNode, depth, visited);
    visited.add(structureNode.id);
    const layout = layoutById.get(structureNode.id)!;
    const nodeWorld = composeTransforms(world, localTransformOf(layout));
    validateContentBounds(layout, nodeWorld);
    const children = index.childrenByParent.get(structureNode.id) ?? [];
    validateGroupContents(structureNode, children.length);
    for (const child of children) queue.push({ structureNode: child, depth: depth + 1, world: nodeWorld });
  }
  if (visited.size !== structure.length)
    throw new Error("Structure contains a cycle or disconnected structure");
}

function validateTraversalPosition(
  structureNode: StructureNode,
  depth: number,
  visited: ReadonlySet<string>,
) {
  if (visited.has(structureNode.id)) throw new Error(`Structure cycle at: ${structureNode.id}`);
  if (depth > MAX_STRUCTURE_DEPTH) throw new Error(`Structure depth exceeds limit: ${structureNode.id}`);
}

function validateContentBounds(layout: StructureLayout, world: Parameters<typeof contentWorldBounds>[1]) {
  if (layout.kind === "content") contentWorldBounds(layout, world);
}

function validateGroupContents(structureNode: StructureNode, childCount: number) {
  if (structureNode.kind === "group" && childCount === 0) throw new Error(`Empty Group: ${structureNode.id}`);
}
