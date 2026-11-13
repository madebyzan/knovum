import { computed, onBeforeUnmount, shallowRef, watch, type Ref } from "vue";

import { availableWhen, canRunAction } from "../../../../../shared/contracts/action-availability";
import { useImageCommentSessionStore } from "../../../comments/image-comment-session-store";
import type { ImageExecutionTarget } from "../../../contracts/generated/ImageExecutionTarget";
import {
  useImageEditSessionStore,
  type ImageEditTool,
} from "../../../image-editing/image-edit-session-store";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import { useImageRemovalSessionStore } from "../../../knowledge-derivations/image-removal-session-store";
import { useImageSelectionSessionStore } from "../../../knowledge-derivations/image-selection-session-store";
import { deriveImageSet } from "../../../knowledge-derivations/image-sets";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { useNodeOperationGate } from "../../use-node-operation-gate";
import { imageToolAvailability, type ImageActionAvailability } from "./image-action-availability";
import { imageActionHelp } from "./image-action-help";
import { DIRECT_DERIVATIONS, type DirectImageDerivation } from "./image-derivation-actions";
import { useImageSourceDimensionsStore } from "./image-source-dimensions-store";
import { useImageLayersChoice } from "./use-image-layers-choice";

type ImagePopover = "upscale";

export type { DirectImageDerivation } from "./image-derivation-actions";

export function useImageNodeBarState(
  node: Readonly<Ref<SpaceCanvasNode<SpaceImagePayload>>>,
  derivations: Readonly<Ref<CreationIntentDecisions>>,
  invokeDerivation: (actionId: CreationIntentId, execution?: ImageExecutionTarget) => void,
) {
  const session = useImageEditSessionStore();
  const imageDimensions = useImageSourceDimensionsStore();
  const imageSelection = useImageSelectionSessionStore();
  const removal = useImageRemovalSessionStore();
  const nodeOperations = useNodeOperationGate();
  const comments = useImageCommentSessionStore();
  const layersChoice = useImageLayersChoice(
    (models) => void deriveImageSet("split-image-layers", node.value, models),
  );
  const openPopover = shallowRef<ImagePopover>();
  const activeTool = computed(() =>
    session.active?.nodeId === node.value.id ? session.active.tool : undefined,
  );
  const commenting = computed(() => comments.activeNodeId === node.value.id);
  const removing = computed(() => removal.active?.nodeId === node.value.id);
  const availabilityByDerivation = computed(() => {
    const idle = !session.active && !removal.active && !commenting.value;
    return Object.fromEntries(
      DIRECT_DERIVATIONS.map((actionId) => [
        actionId,
        canRunAction(derivations.value[actionId])
          ? availableWhen(idle, "finish_image_edit")
          : { state: "hidden" as const },
      ]),
    ) as Record<DirectImageDerivation, ImageActionAvailability>;
  });
  const reasonByDerivation = computed(
    () =>
      Object.fromEntries(
        DIRECT_DERIVATIONS.map((actionId) => [
          actionId,
          imageActionHelp(availabilityByDerivation.value[actionId])?.label ?? "",
        ]),
      ) as Record<DirectImageDerivation, string>,
  );
  const dimensionsReady = computed(() => {
    const loaded = imageDimensions.forNode(node.value.id, node.value.data.sourceRevision);
    return Boolean(
      (node.value.data.intrinsicWidth ?? loaded?.width) &&
      (node.value.data.intrinsicHeight ?? loaded?.height),
    );
  });
  const interactiveAvailability = (busy: boolean) =>
    imageToolAvailability({
      sourceReady: Boolean(node.value.data.sourceRevision),
      editing: !nodeOperations.allows("derive"),
      busy,
    });
  const removalAvailability = computed(() => interactiveAvailability(removal.submitting));
  const removalReason = computed(() => imageActionHelp(removalAvailability.value)?.label ?? "");
  const selectionAvailability = computed(() => interactiveAvailability(imageSelection.submitting));
  const selectionReason = computed(() => imageActionHelp(selectionAvailability.value)?.label ?? "");

  function beginTool(tool: ImageEditTool) {
    if (
      node.value.data.imageDepth ||
      !nodeOperations.allows("edit") ||
      session.applying ||
      (tool === "crop" && !dimensionsReady.value)
    )
      return;
    if (commenting.value) comments.cancel();
    openPopover.value = undefined;
    session.begin(node.value.id, tool);
  }

  function beginRemoval() {
    if (!canRunAction(removalAvailability.value)) return;
    openPopover.value = undefined;
    removal.begin(node.value.id);
  }

  function beginSelection() {
    if (!canRunAction(selectionAvailability.value)) return;
    openPopover.value = undefined;
    void imageSelection.begin(node.value);
  }

  function setPopover(mode: ImagePopover, open: boolean) {
    if (!nodeOperations.allows("derive")) return;
    openPopover.value = open ? mode : openPopover.value === mode ? undefined : openPopover.value;
  }

  function beginAngle() {
    if (!nodeOperations.allows("derive")) return false;
    openPopover.value = undefined;
    return true;
  }

  function beginComment() {
    if (!nodeOperations.allows("edit") || session.applying) return;
    openPopover.value = undefined;
    comments.begin(node.value.id);
  }

  function runDirectDerivation(actionId: DirectImageDerivation, execution?: ImageExecutionTarget) {
    if (!nodeOperations.allows("derive") || !canRunAction(availabilityByDerivation.value[actionId])) return;
    openPopover.value = undefined;
    if (actionId === "split-image-layers") void layersChoice.begin();
    else invokeDerivation(actionId, execution);
  }

  function cancelNodeSessions(nodeId: string | undefined) {
    if (imageSelection.active?.nodeId === nodeId) imageSelection.cancel();
    if (removal.active?.nodeId === nodeId) removal.cancel();
    if (comments.activeNodeId === nodeId) comments.cancel();
  }

  watch(
    () => node.value.id,
    (_next, previous) => {
      openPopover.value = undefined;
      layersChoice.cancel();
      cancelNodeSessions(previous);
    },
  );
  onBeforeUnmount(() => cancelNodeSessions(node.value.id));

  return {
    openPopover,
    layersChoice,
    activeTool,
    removing,
    commenting,
    availabilityByDerivation,
    reasonByDerivation,
    dimensionsReady,
    removalAvailability,
    removalReason,
    selectionAvailability,
    selectionReason,
    beginTool,
    beginRemoval,
    beginSelection,
    setPopover,
    beginAngle,
    beginComment,
    runDirectDerivation,
  };
}
