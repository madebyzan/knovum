import { computed, onBeforeUnmount, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { availableWhen, canRunAction } from "../../../../shared/contracts/action-availability";
import { useNodeOperationGate } from "../../canvas/node-operation-gate";
import { useImageCommentSession } from "../../comments/image-comment-session";
import { useImageEditSession, type ImageEditTool } from "../../image-editing/image-edit-session";
import type { CreationIntentDecisions } from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { imageActionHelp } from "./image-action-help";

type ImagePopover = "upscale" | "angle" | "layers";

export function useImageNodeBarState(
  node: Readonly<Ref<SpaceCanvasNode<SpaceImagePayload>>>,
  derivations: Readonly<Ref<CreationIntentDecisions>>,
  invokeBackgroundRemoval: () => void,
) {
  const { t } = useI18n();
  const session = useImageEditSession();
  const nodeOperations = useNodeOperationGate();
  const comments = useImageCommentSession();
  const openPopover = shallowRef<ImagePopover>();
  const activeTool = computed(() =>
    session.active?.nodeId === node.value.id ? session.active.tool : undefined,
  );
  const commenting = computed(() => comments.activeNodeId === node.value.id);
  const directAvailability = computed(() => {
    const decision = derivations.value["remove-image-background"];
    if (!canRunAction(decision)) return { state: "hidden" as const };
    return availableWhen(!session.active && !commenting.value, "finish_image_edit");
  });
  const directReason = computed(() => imageActionHelp(directAvailability.value, t)?.label ?? "");
  const dimensionsReady = computed(() => {
    const loaded = session.dimensionsFor(node.value.id);
    return Boolean(
      (node.value.data.intrinsicWidth ?? loaded?.width) &&
      (node.value.data.intrinsicHeight ?? loaded?.height),
    );
  });

  function beginTool(tool: ImageEditTool) {
    if (!nodeOperations.allows("edit") || session.applying || (tool === "crop" && !dimensionsReady.value))
      return;
    if (commenting.value) comments.cancel();
    openPopover.value = undefined;
    session.begin(node.value.id, tool);
  }

  function setPopover(mode: ImagePopover, open: boolean) {
    if (!nodeOperations.allows("derive")) return;
    openPopover.value = open ? mode : openPopover.value === mode ? undefined : openPopover.value;
  }

  function beginComment() {
    if (!nodeOperations.allows("edit") || session.applying) return;
    openPopover.value = undefined;
    comments.begin(node.value.id);
  }

  function removeBackground() {
    if (!nodeOperations.allows("derive") || !canRunAction(directAvailability.value)) return;
    openPopover.value = undefined;
    invokeBackgroundRemoval();
  }

  watch(
    () => node.value.id,
    (_next, previous) => {
      openPopover.value = undefined;
      if (session.active?.nodeId === previous) session.cancel();
      if (comments.activeNodeId === previous) comments.cancel();
    },
  );
  onBeforeUnmount(() => {
    if (session.active?.nodeId === node.value.id) session.cancel();
    if (comments.activeNodeId === node.value.id) comments.cancel();
  });

  return {
    comments,
    openPopover,
    activeTool,
    commenting,
    directAvailability,
    directReason,
    dimensionsReady,
    beginTool,
    setPopover,
    beginComment,
    removeBackground,
  };
}
