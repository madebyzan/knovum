import { compact } from "es-toolkit";

import type { CanvasRect, CanvasSelectionSnapshot } from "../contracts/canvas-editor-port";
import type { CanvasNodeLayoutV1 } from "../schema/document";
import { canvasLayoutBounds } from "./canvas-layout-bounds";

interface CanvasSelectionInput {
  layouts: readonly CanvasNodeLayoutV1[];
  nodeIds: ReadonlySet<string>;
  relationIds: ReadonlySet<string>;
  relationBoundsById: ReadonlyMap<string, CanvasRect>;
  structureNodeIds?: ReadonlySet<string>;
  structureBoundsById?: ReadonlyMap<string, CanvasRect>;
}

export function createCanvasSelectionSnapshot({
  layouts,
  nodeIds,
  relationIds,
  relationBoundsById,
  structureNodeIds = new Set(),
  structureBoundsById = new Map(),
}: CanvasSelectionInput): CanvasSelectionSnapshot {
  const selectedLayouts = layouts.filter(({ nodeId }) => nodeIds.has(nodeId));
  const structureMembers = compact([...structureNodeIds].map((id) => structureBoundsById.get(id)));
  const contentBounds = canvasLayoutBounds(selectedLayouts);
  const bounds =
    rectBounds([...(contentBounds ? [contentBounds] : []), ...structureMembers]) ??
    selectedRelationBounds(relationIds, relationBoundsById);
  return {
    nodeIds: [...nodeIds],
    structureNodeIds: [...structureNodeIds],
    nodePlacements: selectedLayouts.map(({ nodeId, frame }) => ({ id: nodeId, x: frame.x, y: frame.y })),
    relationIds: [...relationIds],
    members: [...selectedLayouts.map(({ frame }) => frame), ...structureMembers],
    ...(!structureNodeIds.size && selectedLayouts.length === 1 && selectedLayouts[0]?.appearance
      ? { singleNodeAppearance: selectedLayouts[0].appearance }
      : {}),
    ...(bounds ? { bounds } : {}),
  };
}

function selectedRelationBounds(
  relationIds: ReadonlySet<string>,
  boundsByRelationId: ReadonlyMap<string, CanvasRect>,
) {
  const selected = compact([...relationIds].map((id) => boundsByRelationId.get(id)));
  return rectBounds(selected);
}

function rectBounds(selected: readonly CanvasRect[]) {
  if (!selected.length) return undefined;
  const left = Math.min(...selected.map(({ x }) => x));
  const top = Math.min(...selected.map(({ y }) => y));
  const right = Math.max(...selected.map(({ x, width }) => x + width));
  const bottom = Math.max(...selected.map(({ y, height }) => y + height));
  return { x: left, y: top, width: right - left, height: bottom - top };
}
