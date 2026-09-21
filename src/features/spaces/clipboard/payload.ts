import type { SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import type { StructureLayout, StructureNode } from "../model/structure/structure-contract";
import type { createStructureIndex } from "../model/structure/structure-index";
import { effectiveOperationRoots } from "../model/structure/structure-index";
import { validateStructureSnapshot } from "../model/structure/structure-validation";
import { worldTransformFor } from "../model/structure/world-transform";
import type { SpaceClipboardPayload } from "./payload-types";

export function createSpaceClipboardPayload(input: {
  snapshot: SpaceEditSnapshot;
  nodes: readonly SpaceCanvasNode[];
  relations: readonly SpaceRelation[];
  selectedStructureNodeIds: readonly string[];
}): SpaceClipboardPayload {
  const index = validateStructureSnapshot(input.snapshot.structure, input.snapshot.layouts);
  const layouts = new Map(input.snapshot.layouts.map((layout) => [layout.structureNodeId, layout]));
  const roots = effectiveOperationRoots(input.selectedStructureNodeIds, index);
  const included = collectSubtrees(roots, index);
  const structure = input.snapshot.structure
    .filter(({ id }) => included.has(id))
    .map((structureNode) => detachRoot(structureNode, roots));
  const structureLayouts = input.snapshot.layouts
    .filter(({ structureNodeId }) => included.has(structureNodeId))
    .map((layout) => detachRootLayout(layout, roots, index, layouts));
  validateStructureSnapshot(structure, structureLayouts);
  const nodeIds = new Set(
    structure.flatMap((structureNode) => (structureNode.kind === "content" ? [structureNode.nodeId] : [])),
  );
  const nodes = input.nodes
    .filter(({ id }) => nodeIds.has(id))
    .map(({ id, data }) => ({ id, data: validateSpaceNodePayload(data) }));
  if (nodes.length !== nodeIds.size) throw new Error("Clipboard structure contains unavailable content");
  const relations = input.relations.filter(
    ({ sourceId, targetId }) => nodeIds.has(sourceId) && nodeIds.has(targetId),
  );
  return { version: 1, structure, layouts: structureLayouts, nodes, relations };
}

function collectSubtrees(roots: readonly StructureNode[], index: ReturnType<typeof createStructureIndex>) {
  const ids = new Set<string>();
  const pending = roots.map(({ id }) => id);
  while (pending.length) {
    const id = pending.pop()!;
    if (ids.has(id)) continue;
    ids.add(id);
    for (const child of index.childrenByParent.get(id) ?? []) pending.push(child.id);
  }
  return ids;
}

function detachRoot(structureNode: StructureNode, roots: readonly StructureNode[]): StructureNode {
  const order = roots.findIndex(({ id }) => id === structureNode.id);
  return order < 0 ? structureNode : { ...structureNode, parentId: null, order };
}

function detachRootLayout(
  layout: StructureLayout,
  roots: readonly StructureNode[],
  index: ReturnType<typeof createStructureIndex>,
  layouts: ReadonlyMap<string, StructureLayout>,
): StructureLayout {
  if (!roots.some(({ id }) => id === layout.structureNodeId) || layout.kind === "section") return layout;
  return { ...layout, transform: worldTransformFor(layout.structureNodeId, index, layouts) };
}
