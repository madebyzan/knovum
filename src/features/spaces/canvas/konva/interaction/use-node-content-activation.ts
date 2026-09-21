import { computed, nextTick, onScopeDispose, shallowRef, watch, type Ref } from "vue";

import type { CanvasPoint } from "../../contract/canvas-editor";
import type { CanvasTool } from "../../contract/canvas-tool";
import type { CanvasScene } from "../scene/scene-projection";
import type { useCanvasSelection } from "../selection/use-canvas-selection";

interface NodeContentActivationOptions {
  activate(id: string): Promise<void>;
  create(kind: "note" | "text", point: CanvasPoint): Promise<string | undefined>;
  tool(): CanvasTool;
  reportError(error: unknown): void;
  scene: Readonly<Ref<CanvasScene>>;
  selection: ReturnType<typeof useCanvasSelection>;
}

export function useNodeContentActivation(options: NodeContentActivationOptions) {
  const interactiveNodeId = shallowRef<string>();
  let disposed = false;
  onScopeDispose(() => {
    disposed = true;
  });

  async function createTextNode(kind: "note" | "text", point: CanvasPoint) {
    const id = await options.create(kind, point);
    await nextTick();
    if (id && !disposed && options.tool() === "select") activateNode(id);
  }

  function activateNode(id: string) {
    const sceneNode = options.scene.value.nodes.find(({ node }) => node.id === id);
    if (!sceneNode) return;
    if (sceneNode.interaction.activationMode !== "primary") {
      options.selection.setNodeIds([id]);
      interactiveNodeId.value = id;
      return;
    }
    void options.activate(id).catch(options.reportError);
  }

  watch(
    options.selection.nodeIds,
    (ids) => {
      const selectedId = ids.size === 1 ? ids.values().next().value : undefined;
      if (selectedId) {
        const selected = options.scene.value.nodes.find(({ node }) => node.id === selectedId);
        if (selected?.interaction.activationMode === "select-to-interact") {
          interactiveNodeId.value = selectedId;
        }
      }
      if (interactiveNodeId.value && interactiveNodeId.value !== selectedId) {
        interactiveNodeId.value = undefined;
      }
    },
    { immediate: true },
  );

  const capturesCanvasInteraction = computed(() => {
    if (!interactiveNodeId.value) return false;
    return (
      options.scene.value.nodes.find(({ node }) => node.id === interactiveNodeId.value)?.interaction
        .activationMode === "activate-to-interact"
    );
  });

  return { activateNode, capturesCanvasInteraction, createTextNode, interactiveNodeId };
}
