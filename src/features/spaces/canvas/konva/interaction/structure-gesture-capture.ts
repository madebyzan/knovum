import type Konva from "konva";

import type { SpaceEditSnapshot } from "../../../model/structure/schema/edit-snapshot";
import {
  ancestorsOf,
  createStructureIndex,
  effectiveOperationRoots,
} from "../../../model/structure/structure-index";
import { isStructureNodeEffectivelyLocked } from "../../../model/structure/structure-queries";
import type { CanvasRect } from "../../contracts/canvas-editor-port";
import { createCanvasTranslationSnapIndex } from "./canvas-translation-snap-index";
import { createTranslationSnapSession } from "./translation-snap-session";

interface StructureGestureCaptureInput {
  id: string;
  target: Konva.Node;
  snapshot: SpaceEditSnapshot;
  selectedIds: readonly string[];
  bounds: ReadonlyMap<string, CanvasRect>;
}

export function captureStructureGesture({
  id,
  target,
  snapshot,
  selectedIds,
  bounds,
}: StructureGestureCaptureInput) {
  const index = createStructureIndex(snapshot.structure);
  const selectedAncestor = ancestorsOf(id, index).some((ancestor) => selectedIds.includes(ancestor.id));
  const roots = effectiveOperationRoots(
    selectedIds.includes(id) || selectedAncestor ? selectedIds : [id],
    index,
  )
    .filter(({ id: rootId }) => !isStructureNodeEffectivelyLocked(rootId, index))
    .map(({ id: rootId }) => rootId);
  if (!roots.length) return undefined;
  const moving = new Set(roots);
  const stationary = [...bounds].filter(
    ([rootId]) =>
      !moving.has(rootId) && !ancestorsOf(rootId, index).some((ancestor) => moving.has(ancestor.id)),
  );
  return {
    id,
    snapshot,
    roots,
    target,
    origin: target.position(),
    snap: createTranslationSnapSession(
      roots.map((rootId) => snapFrame(bounds.get(rootId)!)),
      createCanvasTranslationSnapIndex(stationary.map(([, frame]) => snapFrame(frame))),
    ),
  };
}

function snapFrame(frame: CanvasRect) {
  return { x: frame.x, y: frame.y, width: frame.width, height: frame.height, rotation: 0 };
}
