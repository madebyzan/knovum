import { compact } from "es-toolkit";

import type { CanvasRect, CanvasSelectionSnapshot } from "../contracts/canvas-editor-port";
import type { CanvasNodeLayoutV1 } from "../schema/document";
import { canvasLayoutBounds } from "./canvas-layout-bounds";

export function createCanvasSelectionSnapshot(
  layouts: readonly CanvasNodeLayoutV1[],
  nodeIds: ReadonlySet<string>,
  relationIds: ReadonlySet<string>,
  relationBounds: ReadonlyMap<string, CanvasRect>,
): CanvasSelectionSnapshot {
  const selectedLayouts = layouts.filter(({ nodeId }) => nodeIds.has(nodeId));
  const bounds = canvasLayoutBounds(selectedLayouts) ?? selectedRelationBounds(relationIds, relationBounds);
  return {
    nodeIds: [...nodeIds],
    structureNodeIds: [],
    nodePlacements: selectedLayouts.map(({ nodeId, frame }) => ({ id: nodeId, x: frame.x, y: frame.y })),
    relationIds: [...relationIds],
    members: selectedLayouts.map(({ frame }) => frame),
    ...(selectedLayouts.length === 1 && selectedLayouts[0]?.appearance
      ? { singleNodeAppearance: selectedLayouts[0].appearance }
      : {}),
    ...(bounds ? { bounds } : {}),
  };
}

function selectedRelationBounds(ids: ReadonlySet<string>, bounds: ReadonlyMap<string, CanvasRect>) {
  const selected = compact([...ids].map((id) => bounds.get(id)));
  if (!selected.length) return undefined;
  const left = Math.min(...selected.map(({ x }) => x));
  const top = Math.min(...selected.map(({ y }) => y));
  const right = Math.max(...selected.map(({ x, width }) => x + width));
  const bottom = Math.max(...selected.map(({ y, height }) => y + height));
  return { x: left, y: top, width: right - left, height: bottom - top };
}

export function canvasNodePlacements(layouts: readonly CanvasNodeLayoutV1[], nodeIds: ReadonlySet<string>) {
  return layouts.flatMap(({ nodeId, frame }) =>
    nodeIds.has(nodeId) ? [{ id: nodeId, x: frame.x, y: frame.y }] : [],
  );
}

export function movableCanvasNodeIds(lockedIds: ReadonlySet<string>, selectedIds: ReadonlySet<string>) {
  return new Set([...selectedIds].filter((id) => !lockedIds.has(id)));
}
