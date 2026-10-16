import { computed, onBeforeUnmount, shallowRef, watch, type Ref } from "vue";

import { availableWhen, canRunAction } from "../../../../../shared/contracts/action-availability";
import { useImageCommentSessionStore } from "../../../comments/image-comment-session-store";
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
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { useNodeOperationGate } from "../../use-node-operation-gate";
import { imageToolAvailability, type ImageActionAvailability } from "./image-action-availability";
import { imageActionHelp } from "./image-action-help";
import { useImageSourceDimensionsStore } from "./image-source-dimensions-store";

type ImagePopover = "upscale";

/** Image derivations the node bar offers as a single click. */
const DIRECT_DERIVATIONS = ["remove-image-background", "split-image-layers"] as const;
type DirectDerivation = (typeof DIRECT_DERIVATIONS)[number];

export function useImageNodeBarState(
  node: Readonly<Ref<SpaceCanvasNode<SpaceImagePayload>>>,
  derivations: Readonly<Ref<CreationIntentDecisions>>,
  invokeDerivation: (actionId: CreationIntentId) => void,
) {
  const session = useImageEditSessionStore();
  const imageDimensions = useImageSourceDimensionsStore();
  const imageSelection = useImageSelectionSessionStore();
  const removal = useImageRemovalSessionStore();
  const nodeOperations = useNodeOperationGate();
  const comments = useImageCommentSessionStore();
  const openPopover = shallowRef<ImagePopover>();
  const activeTool = computed(() =>
    session.active?.nodeId === node.value.id ? session.active.tool : undefined,
  );
  const commenting = computed(() => comments.activeNodeId === node.value.id);
  const removing = computed(() => removal.active?.nodeId === node.value.id);
  const directAvailability = computed(() => {
    const idle = !session.active && !removal.active && !commenting.value;
    return Object.fromEntries(
      DIRECT_DERIVATIONS.map((actionId) => [
        actionId,
        canRunAction(derivations.value[actionId])
          ? availableWhen(idle, "finish_image_edit")
          : { state: "hidden" as const },
      ]),
    ) as Record<DirectDerivation, ImageActionAvailability>;
  });
  const directReason = computed(
    () =>
      Object.fromEntries(
        DIRECT_DERIVATIONS.map((actionId) => [
          actionId,
          imageActionHelp(directAvailability.value[actionId])?.label ?? "",
        ]),
      ) as Record<DirectDerivation, string>,
  );
  const dimensionsReady = computed(() => {
    const loaded = imageDimensions.forNode(node.value.id, node.value.data.sourceRevision);
    return Boolean(
      (node.value.data.intrinsicWidth ?? loaded?.width) &&
      (node.value.data.intrinsicHeight ?? loaded?.height),
    );
  });
  const removalAvailability = computed(() =>
    imageToolAvailability({
      sourceReady: Boolean(node.value.data.sourceRevision),
      editing: !nodeOperations.allows("derive"),
      busy: removal.submitting,
    }),
  );
  const removalReason = computed(() => imageActionHelp(removalAvailability.value)?.label ?? "");

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

  function runDirectDerivation(actionId: DirectDerivation) {
    if (!nodeOperations.allows("derive") || !canRunAction(directAvailability.value[actionId])) return;
    openPopover.value = undefined;
    invokeDerivation(actionId);
  }

  watch(
    () => node.value.id,
    (_next, previous) => {
      openPopover.value = undefined;
      if (imageSelection.active?.nodeId === previous) imageSelection.cancel();
      if (removal.active?.nodeId === previous) removal.cancel();
      if (comments.activeNodeId === previous) comments.cancel();
    },
  );
  onBeforeUnmount(() => {
    if (imageSelection.active?.nodeId === node.value.id) imageSelection.cancel();
    if (removal.active?.nodeId === node.value.id) removal.cancel();
    if (comments.activeNodeId === node.value.id) comments.cancel();
  });

  return {
    openPopover,
    activeTool,
    removing,
    commenting,
    directAvailability,
    directReason,
    dimensionsReady,
    removalAvailability,
    removalReason,
    beginTool,
    beginRemoval,
    setPopover,
    beginAngle,
    beginComment,
    runDirectDerivation,
  };
}
