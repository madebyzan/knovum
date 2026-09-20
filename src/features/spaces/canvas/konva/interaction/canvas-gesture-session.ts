import type Konva from "konva";
import { nextTick, shallowRef } from "vue";

import type { StructureCommand } from "../../../organization/structure/commands/contract";
import { planStructureCommand } from "../../../organization/structure/commands/plan";
import { resolveSectionDrop } from "../../../organization/structure/section-drop-intent";
import type { CanvasDisplayNode } from "../../contract/canvas-display-node";
import type {
  CanvasNodeFrameUpdate,
  CanvasNodeLayoutV1,
  SpaceEditorDocument,
} from "../../contract/canvas-document";
import type { CanvasSurfaceEvents } from "../../contract/canvas-editor";
import type { CanvasPoint } from "../../contract/canvas-editor-port";
import { replayEditorTransition } from "../../document/editor-history-transition";
import { projectEditorLayouts } from "../../document/editor-layout-projection";
import type { CanvasDocumentModel } from "../document/canvas-document-model";
import { applyCanvasGestureUpdates } from "../document/canvas-document-mutations";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
import type { CanvasGestureFeedback } from "./canvas-snapping";
import { NodeGestureController } from "./node-gesture-controller";
interface Options {
  document: CanvasDocumentModel;
  events: CanvasSurfaceEvents;
  layouts(): readonly CanvasNodeLayoutV1[];
  nodes(): readonly CanvasDisplayNode[];
  previewFrames(updates: readonly CanvasGestureUpdate[]): void;
  selectedNodeIds(): ReadonlySet<string>;
  setSelection(ids: readonly string[]): void;
  stage(): Konva.Stage | undefined;
  zoom(): number;
}
export type CanvasDirectManipulation = "drag" | "transform";
export function createCanvasGestureSession(options: Options) {
  const busy = shallowRef(false),
    directManipulation = shallowRef<CanvasDirectManipulation>();
  const feedback = shallowRef<CanvasGestureFeedback>(),
    dropTarget = shallowRef<string | null>();
  let origin: SpaceEditorDocument | undefined;
  let ids: readonly string[] = [];
  const controller = new NodeGestureController({
    ...options,
    document: () => options.document.document,
    previewFrames: preview,
    updateFeedback: (value) => {
      feedback.value = value;
    },
  });
  function beginDirectManipulation(kind: CanvasDirectManipulation) {
    origin = options.document.document;
    ids = [...options.selectedNodeIds()];
    directManipulation.value = kind;
    if (kind === "transform") controller.beginTransform();
  }
  function preview(updates: readonly CanvasGestureUpdate[]) {
    options.previewFrames(updates);
    if (!origin || !ids.length || directManipulation.value !== "drag") return;
    const point = options.stage()?.getRelativePointerPosition();
    if (point) dropTarget.value = resolveSectionDrop(origin, ids, point, dropTarget.value, options.zoom());
  }
  async function finish(gesture: { kind: "drag"; nodeId: string } | { kind: "transform" }) {
    const before = origin;
    if (!before) return;
    const selected = ids;
    const target = dropTarget.value;
    const updates = gesture.kind === "drag" ? controller.endDrag(gesture.nodeId) : controller.endTransform();
    await tracked(async () => {
      try {
        let after: SpaceEditorDocument;
        if (gesture.kind === "drag") {
          const start = projectEditorLayouts(before).find(
            (layout) => layout.nodeId === gesture.nodeId,
          )?.frame;
          const frame = updates.find((update) => update.nodeId === gesture.nodeId)?.frame;
          if (!start || !frame) return;
          const delta = { x: frame.x - start.x, y: frame.y - start.y };
          const command: StructureCommand =
            target === undefined
              ? { kind: "move-items", ids: [...selected], delta }
              : { kind: "drop-items", ids: [...selected], parentId: target, delta };
          after = { ...before, ...planStructureCommand(before, command).after };
        } else after = applyCanvasGestureUpdates(before, updates);
        await options.document.mutate((current) => replayEditorTransition(current, before, after));
      } finally {
        await nextTick();
        controller.restoreFrames(options.layouts());
        origin = undefined;
        ids = [];
        dropTarget.value = undefined;
        directManipulation.value = undefined;
        feedback.value = undefined;
        options.previewFrames([]);
      }
    });
  }
  async function runStructureCommand(command: StructureCommand) {
    return tracked(async () => {
      const selection = await options.document.execute(command, [...options.selectedNodeIds()]);
      await nextTick();
      options.setSelection(selection);
    });
  }
  async function runFrameMutation(
    calculate: () => readonly CanvasNodeFrameUpdate[] | Promise<readonly CanvasNodeFrameUpdate[]>,
  ) {
    const before = options.document.document;
    const updates = await calculate();
    const layouts = new Map(projectEditorLayouts(before).map((layout) => [layout.nodeId, layout.frame]));
    let after = before;
    for (const update of updates) {
      const frame = layouts.get(update.nodeId);
      if (!frame) throw new Error("Arrangement target disappeared");
      after = {
        ...after,
        ...planStructureCommand(after, {
          kind: "move-items",
          ids: [update.nodeId],
          delta: { x: update.frame.x - frame.x, y: update.frame.y - frame.y },
        }).after,
      };
    }
    await tracked(() => options.document.mutate((current) => replayEditorTransition(current, before, after)));
  }
  function cancelActiveGesture() {
    const cancelled = controller.cancelActiveGesture();
    if (!busy.value) {
      origin = undefined;
      ids = [];
      dropTarget.value = undefined;
      directManipulation.value = undefined;
    }
    return cancelled;
  }
  async function tracked(work: () => Promise<void>) {
    busy.value = true;
    options.events.onMutationPendingChange(true);
    try {
      await work();
    } finally {
      busy.value = false;
      options.events.onMutationPendingChange(false);
    }
  }
  const nudge = (delta: CanvasPoint) =>
    runStructureCommand({ kind: "move-items", ids: [...options.selectedNodeIds()], delta });
  return {
    busy,
    controller,
    directManipulation,
    feedback,
    dropTarget,
    beginDirectManipulation,
    finish,
    runStructureCommand,
    runFrameMutation,
    cancelActiveGesture,
    nudge,
  };
}
