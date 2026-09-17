import ELK from "elkjs/lib/elk-api";
import elkWorkerUrl from "elkjs/lib/elk-worker.min.js?url";

import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../contract/canvas-document";
import type { CanvasAutoLayoutDirection } from "../contract/canvas-editor-port";

let layoutEngine: InstanceType<typeof ELK> | undefined;

export async function autoLayoutCanvas(
  layouts: readonly CanvasNodeLayoutV1[],
  edges: readonly { id: string; sourceId: string; targetId: string }[],
  direction: CanvasAutoLayoutDirection,
): Promise<CanvasNodeFrameUpdate[]> {
  if (layouts.length < 2) return [];
  const selected = new Set(layouts.map(({ nodeId }) => nodeId));
  const origin = {
    x: Math.min(...layouts.map(({ frame }) => frame.x)),
    y: Math.min(...layouts.map(({ frame }) => frame.y)),
  };
  const graph = await engine().layout({
    id: "canvas-selection",
    layoutOptions: {
      "elk.algorithm": "layered",
      "elk.direction": direction,
      "elk.spacing.nodeNode": "48",
      "elk.layered.spacing.nodeNodeBetweenLayers": "72",
    },
    children: layouts.map(({ nodeId, frame }) => ({ id: nodeId, width: frame.width, height: frame.height })),
    edges: edges
      .filter(({ sourceId, targetId }) => selected.has(sourceId) && selected.has(targetId))
      .map(({ id, sourceId, targetId }) => ({ id, sources: [sourceId], targets: [targetId] })),
  });
  const current = new Map(layouts.map((layout) => [layout.nodeId, layout.frame]));
  return (graph.children ?? []).flatMap((node) => {
    const frame = current.get(node.id);
    return node.x === undefined || node.y === undefined || !frame
      ? []
      : [{ nodeId: node.id, frame: { ...frame, x: origin.x + node.x, y: origin.y + node.y } }];
  });
}

function engine() {
  layoutEngine ??= new ELK({ workerUrl: elkWorkerUrl });
  return layoutEngine;
}
