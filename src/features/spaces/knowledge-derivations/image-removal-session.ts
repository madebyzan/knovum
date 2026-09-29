import { defineStore } from "pinia";
import { computed, readonly, shallowRef, watch } from "vue";

import type { MaskStroke } from "../contracts/generated/MaskStroke";
import { useSpaceStore } from "../session/space-store";
import { deriveImage } from "./image-operation";

interface RemovalSelection {
  spaceId: string;
  nodeId: string;
  strokes: MaskStroke[];
}

const MAX_STROKES = 100;
const MAX_POINTS = 10_000;

export const useImageRemovalSession = defineStore("spaces-image-removal-session", () => {
  const space = useSpaceStore();
  const active = shallowRef<RemovalSelection>();
  const brushDiameter = shallowRef(0.045);
  const submitting = shallowRef(false);
  const remainingPoints = computed(
    () =>
      MAX_POINTS -
      (active.value?.strokes.reduce((total, stroke) => total + stroke.points.length / 2, 0) ?? 0),
  );
  const canDraw = computed(() =>
    Boolean(
      active.value &&
      !submitting.value &&
      active.value.strokes.length < MAX_STROKES &&
      remainingPoints.value >= 2,
    ),
  );

  function begin(nodeId: string) {
    if (active.value) return;
    if (!space.activeSpaceId) throw new Error("No active Space for image derivation");
    const node = space.nodes.find(({ id }) => id === nodeId);
    if (!node || node.data.kind !== "image") throw new Error("Selected image is unavailable");
    active.value = { spaceId: space.activeSpaceId, nodeId, strokes: [] };
  }

  function addStroke(points: number[]) {
    const current = active.value;
    if (!current) return;
    if (
      !canDraw.value ||
      points.length < 4 ||
      points.length % 2 !== 0 ||
      points.length / 2 > remainingPoints.value ||
      points.some((point) => !Number.isFinite(point) || point < 0 || point > 1)
    )
      throw new Error("Object removal mask exceeds its limits");
    active.value = {
      ...current,
      strokes: [...current.strokes, { diameter: brushDiameter.value, points }],
    };
  }

  function undoStroke() {
    if (submitting.value) return;
    const current = active.value;
    if (!current) return;
    active.value = { ...current, strokes: current.strokes.slice(0, -1) };
  }

  function cancel() {
    if (submitting.value) return;
    active.value = undefined;
  }

  async function generate() {
    const current = active.value;
    if (!current || current.strokes.length === 0 || submitting.value) return;
    if (space.activeSpaceId !== current.spaceId) throw new Error("Image belongs to another Space");
    const node = space.nodes.find(({ id }) => id === current.nodeId);
    if (!node || node.data.kind !== "image") throw new Error("Selected image is unavailable");
    submitting.value = true;
    let outputCreated = false;
    try {
      await deriveImage(space, node, { kind: "remove-object", strokes: current.strokes }, () => {
        outputCreated = true;
        if (active.value === current) active.value = undefined;
        submitting.value = false;
      });
    } finally {
      if (!outputCreated) submitting.value = false;
    }
  }

  watch(
    () => space.activeSpaceId,
    () => (active.value = undefined),
    { flush: "sync" },
  );
  watch(
    () => space.nodes,
    (nodes) => {
      if (active.value && !nodes.some(({ id }) => id === active.value?.nodeId)) active.value = undefined;
    },
    { flush: "sync" },
  );

  return {
    active: readonly(active),
    brushDiameter: readonly(brushDiameter),
    submitting: readonly(submitting),
    canDraw,
    remainingPoints,
    begin,
    addStroke,
    undoStroke,
    cancel,
    generate,
    setBrushDiameter: (diameter: number) => (brushDiameter.value = diameter),
  };
});
