import { defineStore } from "pinia";
import { readonly, shallowRef } from "vue";

import { useSpaceStore } from "../session/space-store";
import { imageEditOf, type ImageEdit } from "./image-edit";

export type ImageEditTool = "adjust" | "crop" | "transform" | "eraser";

interface ActiveImageEdit {
  nodeId: string;
  tool: ImageEditTool;
  draft: ImageEdit;
}

export const useImageEditSession = defineStore("spaces-image-edit-session", () => {
  const active = shallowRef<ActiveImageEdit>();
  const applying = shallowRef(false);
  const brushDiameter = shallowRef(0.045);
  const cropAspectRatio = shallowRef<number>();
  const imageDimensions = shallowRef(new Map<string, { width: number; height: number }>());

  function setImageDimensions(nodeId: string, width: number, height: number) {
    imageDimensions.value = new Map(imageDimensions.value).set(nodeId, { width, height });
  }

  function dimensionsFor(nodeId: string) {
    return imageDimensions.value.get(nodeId);
  }

  function begin(nodeId: string, tool: ImageEditTool) {
    if (applying.value || active.value) return;
    const node = useSpaceStore().nodes.find(({ id }) => id === nodeId);
    if (!node || node.data.kind !== "image") throw new Error("Selected image is unavailable");
    active.value = { nodeId, tool, draft: imageEditOf(node.data) };
    cropAspectRatio.value = undefined;
  }

  function previewFor(nodeId: string) {
    return active.value?.nodeId === nodeId ? active.value.draft : undefined;
  }

  function update(patch: Partial<ImageEdit>) {
    const current = active.value;
    if (!current || applying.value) return;
    active.value = { ...current, draft: { ...current.draft, ...patch } };
  }

  function addEraserStroke(points: number[]) {
    const current = active.value;
    if (current?.tool !== "eraser" || points.length < 4) return;
    update({
      eraserStrokes: [...current.draft.eraserStrokes, { diameter: brushDiameter.value, points }],
    });
  }

  function undoEraserStroke() {
    const current = active.value;
    if (current?.tool !== "eraser") return;
    update({ eraserStrokes: current.draft.eraserStrokes.slice(0, -1) });
  }

  function cancel() {
    if (applying.value) return;
    active.value = undefined;
    cropAspectRatio.value = undefined;
  }

  async function apply() {
    const current = active.value;
    if (!current || applying.value) return;
    applying.value = true;
    try {
      const space = useSpaceStore();
      const node = space.nodes.find(({ id }) => id === current.nodeId);
      if (!node || node.data.kind !== "image") throw new Error("Selected image is unavailable");
      if (JSON.stringify(imageEditOf(node.data)) === JSON.stringify(current.draft)) {
        active.value = undefined;
        return;
      }
      await space.updateManyData([
        {
          id: node.id,
          before: node.data,
          after: { ...node.data, imageEdit: current.draft, visualKnowledge: undefined },
        },
      ]);
      if (active.value === current) active.value = undefined;
      cropAspectRatio.value = undefined;
    } finally {
      applying.value = false;
    }
  }

  return {
    active: readonly(active),
    applying: readonly(applying),
    brushDiameter: readonly(brushDiameter),
    cropAspectRatio: readonly(cropAspectRatio),
    setImageDimensions,
    dimensionsFor,
    begin,
    previewFor,
    update,
    addEraserStroke,
    undoEraserStroke,
    setBrushDiameter: (diameter: number) => (brushDiameter.value = diameter),
    setCropAspectRatio: (ratio?: number) => {
      if (active.value?.tool === "crop" && !applying.value) cropAspectRatio.value = ratio;
    },
    cancel,
    apply,
  };
});
