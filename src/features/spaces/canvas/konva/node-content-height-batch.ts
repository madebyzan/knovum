import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { CanvasContentHeightMeasurement } from "../contracts/canvas-editor-port";
import { spaceNodeMinimumContentHeight, spaceNodeUsesContentFitHeight } from "../node-size";
import type { CanvasNodeLayoutV1, CanvasNodeSizingMode } from "../schema/document";

export interface FittedContentHeight {
  contentRevision: string | null;
  height: number;
  measuredWidth: number;
  sizingMode: CanvasNodeSizingMode;
  typographyScale: number;
}

export function createNodeContentHeightBatch(
  nodeById: (nodeId: string) => SpaceCanvasNode | undefined,
  layoutByNodeId: (nodeId: string) => CanvasNodeLayoutV1 | undefined,
  commit: (heights: ReadonlyMap<string, FittedContentHeight>) => void,
) {
  const measurementByNodeId = new Map<string, CanvasContentHeightMeasurement>();
  let paused = false;
  let queued = false;

  function fit(nodeId: string, measurement: CanvasContentHeightMeasurement) {
    measurementByNodeId.set(nodeId, measurement);
    schedule();
  }

  function setPaused(value: boolean) {
    paused = value;
    schedule();
  }

  function schedule() {
    if (paused || queued || measurementByNodeId.size === 0) return;
    queued = true;
    queueMicrotask(flush);
  }

  function flush() {
    queued = false;
    if (paused) return false;
    const fittedHeightByNodeId = new Map<string, FittedContentHeight>();
    for (const [id, measurement] of measurementByNodeId) {
      const fittedMeasurement = fittedHeight(id, measurement);
      if (fittedMeasurement) fittedHeightByNodeId.set(id, fittedMeasurement);
    }
    measurementByNodeId.clear();
    if (fittedHeightByNodeId.size) commit(fittedHeightByNodeId);
    return true;
  }

  function fittedHeight(
    nodeId: string,
    measurement: CanvasContentHeightMeasurement,
  ): FittedContentHeight | undefined {
    const node = nodeById(nodeId);
    const layout = layoutByNodeId(nodeId);
    if (!node || !layout || !spaceNodeUsesContentFitHeight(node.data)) return undefined;
    const matchesSource = [
      (node.data.knowledgeRevision ?? null) === measurement.contentRevision,
      layout.sizingMode === measurement.sizingMode,
      layout.frame.width === measurement.width,
      (layout.appearance?.typographyScale ?? 1) === measurement.typographyScale,
    ].every(Boolean);
    if (!matchesSource) return undefined;
    const height = Math.max(
      spaceNodeMinimumContentHeight(node.data, measurement.typographyScale),
      measurement.height,
    );
    if (layout.sizingMode === "frame" && layout.frame.height === height) return undefined;
    return {
      contentRevision: measurement.contentRevision,
      height,
      measuredWidth: measurement.width,
      sizingMode: measurement.sizingMode,
      typographyScale: measurement.typographyScale,
    };
  }

  return { fit, flush, setPaused };
}
