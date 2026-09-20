import { indexStructure, structureSubtree } from "../../../model/structure/structure-index";
import type { SpaceEditorDocument, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import {
  composeStructureTransforms,
  invertStructureTransform,
  normalizeStructureRotation,
  transformStructurePoint,
} from "../../document/structure-transform";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
/** Changes only the visible projection during a Group transform; no command/schema round trip per frame. */
export function expandAggregatePreview(
  document: SpaceEditorDocument,
  layouts: readonly CanvasNodeLayoutV1[],
  updates: readonly CanvasGestureUpdate[],
) {
  const tree = indexStructure(document.structure);
  const originals = new Map(layouts.map((layout) => [layout.nodeId, layout]));
  const expanded = new Map(updates.map((update) => [update.nodeId, update]));
  for (const update of updates) {
    if (tree.items.get(update.nodeId)?.kind !== "group") continue;
    const original = originals.get(update.nodeId)!;
    const scale = update.frame.width / original.frame.width;
    const transform = composeStructureTransforms(
      { x: update.frame.x, y: update.frame.y, rotation: update.frame.rotation, scale },
      invertStructureTransform({
        x: original.frame.x,
        y: original.frame.y,
        rotation: original.frame.rotation,
        scale: 1,
      }),
    );
    for (const item of structureSubtree(tree, [update.nodeId])) {
      if (item.id === update.nodeId) continue;
      const layout = originals.get(item.id)!;
      expanded.set(item.id, {
        nodeId: item.id,
        contentScale: (layout.contentScale ?? 1) * scale,
        frame: {
          ...transformStructurePoint(transform, layout.frame),
          width: layout.frame.width * scale,
          height: layout.frame.height * scale,
          rotation: normalizeStructureRotation(layout.frame.rotation + transform.rotation),
        },
      });
    }
  }
  return [...expanded.values()];
}
