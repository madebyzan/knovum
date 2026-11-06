import { sumBy } from "es-toolkit";
import { defineStore } from "pinia";
import { computed, readonly, shallowReactive, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/instance";
import type { ImageExecutionTarget } from "../contracts/generated/ImageExecutionTarget";
import type { ImageOperationTargetOption } from "../contracts/generated/ImageOperationTargetOption";
import type { ImageSelection } from "../contracts/generated/ImageSelection";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
import { useSpaceStore } from "../session/space-store";
import { buildSelectionDerivation, loadSelectionChoices } from "./image-selection-choices";
import { deriveImageSelection } from "./image-selection-derivation";
import { pinSource, releaseWhenSourceChanges, type SourcePin } from "./source-pin";
import { useImageSelectionPreview } from "./use-image-selection-preview";

export type ImageSelectionTool = "include" | "exclude" | "box" | "brush";
interface SelectionDraft {
  sessionId: symbol;
  nodeId: string;
  pin: SourcePin;
  selection: ImageSelection;
  /** The selection model that draws the mask. */
  execution: ImageExecutionTarget | undefined;
  /** The repair model that paints the selected area away. */
  repair: ImageExecutionTarget | undefined;
}

export const useImageSelectionSessionStore = defineStore("spaces-image-selection-session", () => {
  const space = useSpaceStore();
  const active = shallowRef<SelectionDraft>();
  const history = shallowReactive<ImageSelection[]>([]);
  const tool = shallowRef<ImageSelectionTool>("include");
  const brushDiameter = shallowRef(0.045);
  const targets = shallowRef<ImageOperationTargetOption[]>([]);
  const repairTargets = shallowRef<ImageOperationTargetOption[]>([]);
  const { maskSource, busy, error, queue, clear } = useImageSelectionPreview(() => {
    const draft = active.value;
    const node = space.nodes.find(({ id }) => id === draft?.nodeId);
    return draft?.execution && node
      ? { node, spaceId: draft.pin.spaceId, execution: draft.execution, selection: draft.selection }
      : undefined;
  });
  const submitting = shallowRef(false);
  const remainingPoints = computed(
    () => 10_000 - sumBy(active.value?.selection.strokes ?? [], (stroke) => stroke.points.length / 2),
  );
  const canDraw = computed(
    () =>
      !!active.value &&
      tool.value === "brush" &&
      !submitting.value &&
      active.value.selection.strokes.length < 100 &&
      remainingPoints.value >= 2,
  );
  const canSubmit = computed(() => Boolean(maskSource.value && !busy.value && !submitting.value));
  const canRepair = computed(
    () => canSubmit.value && !!active.value && !!buildSelectionDerivation(active.value, "repair"),
  );
  const canUndo = computed(() => Boolean(history.length && !submitting.value));

  function update(selection: ImageSelection) {
    if (!active.value || submitting.value) return;
    history.push(active.value.selection);
    active.value = { ...active.value, selection };
    queue();
  }
  async function begin(node: SpaceCanvasNode<SpaceImagePayload>) {
    if (active.value || !space.activeSpaceId || !node.data.sourceRevision) return;
    active.value = {
      sessionId: Symbol(),
      nodeId: node.id,
      pin: pinSource(space.activeSpaceId, node),
      selection: { points: [], boxRect: null, strokes: [] },
      execution: undefined,
      repair: undefined,
    };
    const draft = active.value;
    history.length = 0;
    tool.value = "include";
    clear();
    try {
      const choices = await loadSelectionChoices();
      if (active.value?.sessionId !== draft.sessionId) return;
      targets.value = choices.selection;
      repairTargets.value = choices.repair;
      active.value = { ...active.value, execution: choices.selectionModel, repair: choices.repairModel };
      queue();
    } catch (cause) {
      if (active.value?.sessionId === draft.sessionId) error.value = getErrorMessage(cause);
    }
  }
  function setExecution(execution: ImageExecutionTarget) {
    if (!active.value || submitting.value) return;
    clear();
    active.value = { ...active.value, execution };
    queue();
  }
  function setRepair(repair: ImageExecutionTarget) {
    if (!active.value || submitting.value) return;
    active.value = { ...active.value, repair };
  }
  function addPoint(x: number, y: number) {
    const selection = active.value?.selection;
    if (!selection || submitting.value) return;
    if (selection.points.length + (selection.boxRect ? 2 : 0) >= 6) {
      error.value = i18n.global.t("spaces.media.imageEditor.selection.promptLimit");
      return;
    }
    update({ ...selection, points: [...selection.points, { x, y, foreground: tool.value === "include" }] });
  }
  function setBox(boxRect: ImageSelection["boxRect"]) {
    const selection = active.value?.selection;
    if (!selection || selection.points.length > 4) return;
    update({ ...selection, boxRect });
  }
  function addStroke(points: number[]) {
    const selection = active.value?.selection;
    if (!selection || !canDraw.value || points.length < 4 || points.length / 2 > remainingPoints.value)
      return;
    update({ ...selection, strokes: [...selection.strokes, { diameter: brushDiameter.value, points }] });
  }
  function undo() {
    if (!active.value || !canUndo.value) return;
    active.value = { ...active.value, selection: history.pop()! };
    queue();
  }
  function cancel() {
    if (submitting.value) return;
    clear();
    history.length = 0;
    active.value = undefined;
  }
  async function submit(effect: "extract" | "repair") {
    const current = active.value;
    const input = current && canSubmit.value ? buildSelectionDerivation(current, effect) : undefined;
    const node = space.nodes.find(({ id }) => id === current?.nodeId);
    if (!input || !node) return;
    submitting.value = true;
    try {
      const run = await deriveImageSelection(node, input);
      if (run) {
        submitting.value = false;
        cancel();
      }
    } finally {
      submitting.value = false;
    }
  }
  // A draft that outlived its source while submitting is released once the submission settles.
  releaseWhenSourceChanges({
    pinned: () => active.value?.pin,
    live: () =>
      pinSource(
        space.activeSpaceId ?? "",
        space.nodes.find(({ id }) => id === active.value?.nodeId),
      ),
    release: cancel,
    retryWhen: [submitting],
  });

  return {
    active: readonly(active),
    tool,
    brushDiameter,
    targets: readonly(targets),
    repairTargets: readonly(repairTargets),
    maskSource: readonly(maskSource),
    busy: readonly(busy),
    submitting: readonly(submitting),
    error: readonly(error),
    canDraw,
    remainingPoints,
    canSubmit,
    canRepair,
    canUndo,
    begin,
    setExecution,
    setRepair,
    addPoint,
    setBox,
    addStroke,
    undo,
    cancel,
    submit,
  };
});
