import type Konva from "konva";
import { nextTick, onScopeDispose, shallowRef, type Ref } from "vue";

import type { SpaceEditCommand } from "../../../contracts/generated/SpaceEditCommand";
import type { SpaceEditSnapshot } from "../../../model/structure/schema/edit-snapshot";
import { createStructureIndex } from "../../../model/structure/structure-index";
import { isStructureNodeEffectivelyLocked } from "../../../model/structure/structure-queries";
import { sectionDropMoveCommand } from "../../../structure/section-drop-reparent";
import type { CanvasPoint, CanvasRect } from "../../contracts/canvas-editor-port";
import type { CanvasDirectManipulation } from "./canvas-gesture-session";
import type { CanvasGestureFeedback } from "./canvas-snapping";
import { snapCanvasTranslation } from "./canvas-translation-snapping";
import { createSectionFrameGesture } from "./section-frame-gesture";
import { captureStructureGesture } from "./structure-gesture-capture";
import {
  projectStructureGesture,
  structureGestureChangesLayout,
  type SectionResizeCorner,
  type StructureGestureCommand,
} from "./structure-gesture-preview";

interface StructureGestureOptions {
  snapshot(): SpaceEditSnapshot;
  selectedRootIds(): readonly string[];
  bounds(): ReadonlyMap<string, CanvasRect>;
  execute(command: SpaceEditCommand): Promise<unknown>;
  preview: Ref<SpaceEditSnapshot | undefined>;
  previewDrop(rootIds: readonly string[], delta: CanvasPoint | undefined): void;
  expandedSectionId(): string | undefined;
  zoom(): number;
}

export function useStructureGestureSession(options: StructureGestureOptions) {
  const directManipulation = shallowRef<CanvasDirectManipulation>();
  const feedback = shallowRef<CanvasGestureFeedback>();
  let active: ReturnType<typeof capture>;
  let command: StructureGestureCommand | undefined;
  let committing = false;

  function capture(id: string, target: Konva.Node, kind: CanvasDirectManipulation) {
    if (committing) return undefined;
    const captured = captureStructureGesture({
      id,
      target,
      snapshot: options.snapshot(),
      selectedIds: options.selectedRootIds(),
      bounds: options.bounds(),
    });
    if (captured) directManipulation.value = kind;
    command = undefined;
    return captured;
  }

  function beginMove(id: string, target: Konva.Node) {
    active = capture(id, target, "drag");
  }

  function move() {
    if (!active) return;
    const delta = { x: active.target.x() - active.origin.x, y: active.target.y() - active.origin.y };
    const snapped = snapCanvasTranslation(active.frames, active.snapIndex, delta, 6 / options.zoom());
    active.target.position({ x: active.origin.x + snapped.x, y: active.origin.y + snapped.y });
    preview({ kind: "structure.move", rootIds: active.roots, delta: { x: snapped.x, y: snapped.y } });
    feedback.value = { guides: snapped.guides };
    options.previewDrop(active.roots, { x: snapped.x, y: snapped.y });
  }

  let resizeGesture: ReturnType<typeof createSectionFrameGesture> | undefined;
  function beginResize(id: string, corner: SectionResizeCorner, target: Konva.Node, origin: CanvasPoint) {
    active = capture(id, target, "transform");
    if (active)
      resizeGesture = createSectionFrameGesture({
        id,
        corner,
        target,
        origin,
        frame: options.bounds().get(id)!,
      });
  }

  function resize() {
    if (active && resizeGesture) preview(resizeGesture.command());
  }

  function beginTransform(id: string, target: Konva.Node) {
    active = capture(id, target, "transform");
  }

  function transform(input: { anchor: CanvasPoint; rotationDelta: number; scaleFactor: number }) {
    if (active)
      preview({ kind: "structure.transform", rootIds: active.roots, ...input, translation: { x: 0, y: 0 } });
  }

  function preview(next: StructureGestureCommand) {
    if (!active) return;
    command = next;
    options.preview.value = projectStructureGesture(active.snapshot, next);
  }

  async function finish() {
    if (!active) return;
    const finished = active;
    const final = command;
    const resizing = resizeGesture;
    active = undefined;
    resizeGesture = undefined;
    feedback.value = undefined;
    if (!final || !structureGestureChangesLayout(finished.snapshot, final)) {
      resizing?.cancel();
      finished.target.position(finished.origin);
      return clear();
    }
    committing = true;
    try {
      const persisted =
        final.kind === "structure.move"
          ? sectionDropMoveCommand(
              { snapshot: finished.snapshot, rootIds: final.rootIds, delta: final.delta },
              options.expandedSectionId(),
            )
          : final;
      await options.execute(persisted);
    } catch (error) {
      finished.target.position(finished.origin);
      throw error;
    } finally {
      await nextTick();
      resizing?.settle(options.snapshot());
      committing = false;
      clear();
    }
  }

  function clear() {
    command = undefined;
    options.preview.value = undefined;
    directManipulation.value = undefined;
    options.previewDrop([], undefined);
  }

  function cancel() {
    if (!active) return false;
    const cancelled = active;
    active = undefined;
    resizeGesture?.cancel();
    resizeGesture = undefined;
    cancelled.target.stopDrag();
    cancelled.target.position(cancelled.origin);
    feedback.value = undefined;
    clear();
    return true;
  }

  function pan(delta: CanvasPoint) {
    if (!active) return;
    active.target.position({ x: active.target.x() + delta.x, y: active.target.y() + delta.y });
    move();
  }

  async function nudge(delta: CanvasPoint) {
    const index = createStructureIndex(options.snapshot().structure);
    const rootIds = options.selectedRootIds().filter((id) => !isStructureNodeEffectivelyLocked(id, index));
    if (rootIds.length) await options.execute({ kind: "structure.move", rootIds, delta });
  }

  onScopeDispose(cancel);
  return {
    beginMove,
    beginResize,
    beginTransform,
    cancel,
    directManipulation,
    feedback,
    finish,
    move,
    nudge,
    pan,
    resize,
    transform,
  };
}
