import { computed, nextTick, onScopeDispose, shallowRef, watch, type Ref } from "vue";

import type { CanvasPoint } from "../../contracts/canvas-editor-port";
import type { CanvasTool } from "../../contracts/canvas-tool";
import type { CanvasScene } from "../scene/canvas-scene";
import type { useCanvasSelection } from "../selection/use-canvas-selection";

interface NodeContentActivationOptions {
  activate(nodeId: string): Promise<void>;
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

  function activateNode(nodeId: string) {
    const sceneNode = options.scene.value.nodes.find(({ node }) => node.id === nodeId);
    if (!sceneNode) return;
    if (sceneNode.interaction.activationMode !== "primary") {
      options.selection.setNodeIds([nodeId]);
      interactiveNodeId.value = nodeId;
      return;
    }
    void options.activate(nodeId).catch(options.reportError);
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
