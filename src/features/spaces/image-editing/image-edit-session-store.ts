import { isEqual } from "es-toolkit";
import { defineStore } from "pinia";
import { computed, readonly, shallowRef, watch } from "vue";

import { useSpaceStore } from "../session/space-store";
import { imageEditOf, type ImageEdit } from "./image-edit";

export type ImageEditTool = "adjust" | "crop" | "transform";

interface ActiveImageEdit {
  spaceId: string;
  nodeId: string;
  tool: ImageEditTool;
  draft: ImageEdit;
}

export const useImageEditSessionStore = defineStore("spaces-image-edit-session", () => {
  const space = useSpaceStore();
  const active = shallowRef<ActiveImageEdit>();
  const applying = shallowRef(false);
  let applyPromise: Promise<void> | undefined;
  const cropAspectRatio = shallowRef<number>();
  const dirty = computed(() => {
    const current = active.value;
    if (!current) return false;
    const node = space.nodes.find(({ id }) => id === current.nodeId);
    return !node || node.data.kind !== "image" || !isEqual(imageEditOf(node.data), current.draft);
  });

  function begin(nodeId: string, tool: ImageEditTool) {
    if (applying.value || active.value) return;
    const node = space.nodes.find(({ id }) => id === nodeId);
    if (!node || node.data.kind !== "image") throw new Error("Selected image is unavailable");
    if (!space.activeSpaceId) throw new Error("No active Space for image editing");
    active.value = {
      spaceId: space.activeSpaceId,
      nodeId,
      tool,
      draft: imageEditOf(node.data),
    };
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

  function cancel() {
    if (applying.value) return;
    active.value = undefined;
    cropAspectRatio.value = undefined;
  }

  function apply(): Promise<void> {
    if (applyPromise) return applyPromise;
    const current = active.value;
    if (!current) return Promise.resolve();
    applying.value = true;
    applyPromise = (async () => {
      if (space.activeSpaceId !== current.spaceId) throw new Error("Image edit belongs to another Space");
      const node = space.nodes.find(({ id }) => id === current.nodeId);
      if (!node || node.data.kind !== "image") throw new Error("Selected image is unavailable");
      if (isEqual(imageEditOf(node.data), current.draft)) {
        active.value = undefined;
        return;
      }
      await space.updateNodeContents([
        {
          id: node.id,
          before: node.data,
          after: { ...node.data, imageEdit: current.draft, visualKnowledge: undefined },
        },
      ]);
      if (active.value === current) active.value = undefined;
      cropAspectRatio.value = undefined;
    })().finally(() => {
      applying.value = false;
      applyPromise = undefined;
    });
    return applyPromise;
  }

  watch(
    () => space.nodes,
    (nodes) => {
      const ids = new Set(nodes.map(({ id }) => id));
      if (active.value && !ids.has(active.value.nodeId)) active.value = undefined;
    },
    { flush: "sync" },
  );

  return {
    active: readonly(active),
    applying: readonly(applying),
    cropAspectRatio: readonly(cropAspectRatio),
    dirty,
    begin,
    previewFor,
    update,
    setCropAspectRatio: (ratio?: number) => {
      if (active.value?.tool === "crop" && !applying.value) cropAspectRatio.value = ratio;
    },
    cancel,
    apply,
    settleApply: () => applyPromise ?? Promise.resolve(),
  };
});
