import { renderRevision } from "../../knowledge/model/revision";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { CanvasContentHeightMeasurement } from "../contract/canvas-editor-port";
import type { EditorTextMeasurement } from "../contract/editor-command";
import { spaceNodeMinimumContentHeight, spaceNodeUsesContentFitHeight } from "../node-size";
import type { CanvasDocumentModel } from "./document/canvas-document-model";

export function createNodeContentHeightBatch(
  model: CanvasDocumentModel,
  nodeById: (id: string) => SpaceCanvasNode | undefined,
) {
  const measurements = new Map<string, CanvasContentHeightMeasurement>();
  let paused = false;
  let queued = false;

  function fit(nodeId: string, measurement: CanvasContentHeightMeasurement) {
    measurements.set(nodeId, measurement);
    schedule();
  }

  function setPaused(value: boolean) {
    paused = value;
    schedule();
  }

  function schedule() {
    if (paused || queued || measurements.size === 0) return;
    queued = true;
    queueMicrotask(flush);
  }

  function flush() {
    queued = false;
    if (paused) return false;
    const fitted: EditorTextMeasurement[] = [];
    for (const [id, measurement] of measurements) {
      const node = nodeById(id);
      const layout = model.layout(id);
      if (
        !node ||
        !layout ||
        !spaceNodeUsesContentFitHeight(node.data) ||
        Math.abs(layout.frame.width / (layout.contentScale ?? 1) - measurement.width) > 0.01 ||
        (layout.appearance?.typographyScale ?? 1) !== measurement.typographyScale
      ) {
        continue;
      }
      fitted.push({
        itemId: id,
        width: measurement.width,
        typographyScale: measurement.typographyScale,
        height: Math.max(
          spaceNodeMinimumContentHeight(node.data, measurement.typographyScale),
          measurement.height,
        ),
        contentRevision: renderRevision(node.data),
      });
    }
    measurements.clear();
    model.fitNodeHeights(fitted);
    return true;
  }

  return { fit, flush, setPaused };
}
