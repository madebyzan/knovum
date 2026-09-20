import type Konva from "konva";

import {
  indexStructure,
  structureSubtree,
  structureAncestors,
} from "../../../model/structure/structure-index";
import {
  assertStructureEditable,
  structureOperationRoots,
} from "../../../organization/structure/structure-selection";
import type {
  CanvasNodeFrameUpdate,
  CanvasNodeLayoutV1,
  SpaceEditorDocument,
} from "../../contract/canvas-document";
import { captureCanvasNodeRefs } from "./canvas-node-access";
import {
  createCanvasTranslationSnapIndex,
  type CanvasTranslationGestureState,
} from "./canvas-translation-snapping";
export function captureCanvasNodeDragState(
  layouts: readonly CanvasNodeLayoutV1[],
  document: SpaceEditorDocument,
  selectedIds: ReadonlySet<string>,
  primaryId: string,
  stage: Konva.Stage | undefined,
) {
  const tree = indexStructure(document.structure);
  const roots = structureOperationRoots(tree, [...selectedIds, primaryId]);
  assertStructureEditable(
    tree,
    roots.map((item) => item.id),
  );
  const affected = new Set(
    structureSubtree(
      tree,
      roots.map((item) => item.id),
    ).map((item) => item.id),
  );
  const ancestors = new Set(
    roots.flatMap((item) =>
      structureAncestors(tree, item.id)
        .filter((parent) => parent.kind === "group")
        .map((parent) => parent.id),
    ),
  );
  const frames = new Map(
    layouts
      .filter((layout) => affected.has(layout.nodeId))
      .map((layout) => [layout.nodeId, { ...layout.frame }]),
  );
  const origin = frames.get(primaryId);
  if (!origin) return undefined;
  const state: CanvasTranslationGestureState = {
    primaryId,
    origin,
    frames,
    snapIndex: createCanvasTranslationSnapIndex(
      layouts
        .filter((layout) => !affected.has(layout.nodeId) && !ancestors.has(layout.nodeId))
        .map((layout) => layout.frame),
    ),
  };
  return { state, nodeRefs: captureCanvasNodeRefs(stage, frames.keys()) };
}
export function restoreCanvasNodeFrames(
  updates: readonly CanvasNodeFrameUpdate[],
  nodes: ReadonlyMap<string, Konva.Node>,
) {
  for (const { nodeId, frame } of updates) {
    const node = nodes.get(nodeId);
    node?.position({ x: frame.x, y: frame.y });
    node?.rotation(frame.rotation);
    node?.scale({ x: 1, y: 1 });
  }
}
