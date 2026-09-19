import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

export interface CanvasMovementScope {
  readonly rootIds: ReadonlySet<string>;
  readonly affectedIds: ReadonlySet<string>;
  readonly ownerById: ReadonlyMap<string, string>;
  readonly blockedIds: readonly string[];
}

/** Selection, operation roots and affected content are deliberately different sets. */
export function resolveCanvasMovementScope(
  nodes: readonly SpaceCanvasNode[],
  selectedIds: ReadonlySet<string>,
): CanvasMovementScope {
  const byId = new Map(nodes.map((node) => [node.id, node]));
  const selected = [...selectedIds].map((id) => {
    const node = byId.get(id);
    if (!node) throw new Error(`The selected canvas item is unavailable: ${id}`);
    return node;
  });
  const containers = new Set(selected.filter((node) => node.data.kind === "section").map(({ id }) => id));
  const roots = selected.filter((node) => !node.data.sectionId || !containers.has(node.data.sectionId));
  const rootIds = new Set(roots.map(({ id }) => id));
  const ownerById = new Map(roots.map(({ id }) => [id, id]));
  for (const node of nodes) {
    if (node.data.kind !== "section" && node.data.sectionId && containers.has(node.data.sectionId)) {
      ownerById.set(node.id, node.data.sectionId);
    }
  }
  // A locked descendant follows its unlocked container. A directly edited locked
  // root (or a root inside a locked Section) blocks the complete operation.
  const blockedIds = roots
    .filter((node) => {
      const parent = node.data.sectionId ? byId.get(node.data.sectionId) : undefined;
      return node.data.locked || parent?.data.locked;
    })
    .map(({ id }) => id);
  return { rootIds, affectedIds: new Set(ownerById.keys()), ownerById, blockedIds };
}

export function requireMovableScope(scope: CanvasMovementScope) {
  if (scope.blockedIds.length) {
    throw new Error("Unlock the selected items or their Section before moving the selection");
  }
  return scope;
}
