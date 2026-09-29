import { computed, onBeforeUnmount, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { availableWhen, canRunAction } from "../../../../shared/contracts/action-availability";
import { useNodeOperationGate } from "../../canvas/node-operation-gate";
import { useImageCommentSession } from "../../comments/image-comment-session";
import { useImageEditSession, type ImageEditTool } from "../../image-editing/image-edit-session";
import type { CreationIntentDecisions } from "../../knowledge-derivations/catalog/creation-intent-definitions";
import { useImageOperationCapabilities } from "../../knowledge-derivations/image-capabilities";
import { useImageRemovalSession } from "../../knowledge-derivations/image-removal-session";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { imageActionHelp } from "./image-action-help";
import { imageRemovalSize, imageRemovalSizeValid, useImageSourceDimensions } from "./image-source-dimensions";

type ImagePopover = "upscale" | "layers" | "prompt-removal";

export function useImageNodeBarState(
  node: Readonly<Ref<SpaceCanvasNode<SpaceImagePayload>>>,
  derivations: Readonly<Ref<CreationIntentDecisions>>,
  invokeBackgroundRemoval: () => void,
) {
  const { t } = useI18n();
  const session = useImageEditSession();
  const imageDimensions = useImageSourceDimensions();
  const removal = useImageRemovalSession();
  const capabilities = useImageOperationCapabilities(computed(() => Boolean(node.value.data.sourceRevision)));
  const nodeOperations = useNodeOperationGate();
  const comments = useImageCommentSession();
  const openPopover = shallowRef<ImagePopover>();
  const activeTool = computed(() =>
    session.active?.nodeId === node.value.id ? session.active.tool : undefined,
  );
  const commenting = computed(() => comments.activeNodeId === node.value.id);
  const removing = computed(() => removal.active?.nodeId === node.value.id);
  const directAvailability = computed(() => {
    const decision = derivations.value["remove-image-background"];
    if (!canRunAction(decision)) return { state: "hidden" as const };
    return availableWhen(!session.active && !removal.active && !commenting.value, "finish_image_edit");
  });
  const directReason = computed(() => imageActionHelp(directAvailability.value, t)?.label ?? "");
  const dimensionsReady = computed(() => {
    const loaded = imageDimensions.forNode(node.value.id, node.value.data.sourceRevision);
    return Boolean(
      (node.value.data.intrinsicWidth ?? loaded?.width) &&
      (node.value.data.intrinsicHeight ?? loaded?.height),
    );
  });
  const removalSizeValid = computed(() =>
    imageRemovalSizeValid(imageRemovalSize(node.value, imageDimensions)),
  );
  const removalMode = computed(() => {
    if (capabilities.data.value?.maskedRemoval.state === "ready" && removalSizeValid.value) return "mask";
    if (capabilities.data.value?.promptRemoval.state === "ready") return "prompt";
    return undefined;
  });
  const removalReason = computed(() => {
    if (!node.value.data.sourceRevision) return t("spaces.media.imageEditor.status.imagePreparing");
    if (capabilities.isFetching.value) return t("spaces.media.imageEditor.status.capabilityChecking");
    if (capabilities.isError.value) return t("spaces.media.imageEditor.errors.capabilityCheckFailed");
    const available = capabilities.data.value;
    if (available?.maskedRemoval.state === "ready" && !removalSizeValid.value)
      return t(
        dimensionsReady.value
          ? "spaces.media.imageEditor.removeObject.sizeLimit"
          : "spaces.media.imageEditor.status.imagePreparing",
      );
    const blocked = [available?.maskedRemoval, available?.promptRemoval].find(
      (capability) => capability?.state === "blocked" && capability.reason !== "provider_unsupported",
    );
    return imageActionHelp(blocked ?? { state: "blocked", reason: "provider_unsupported" }, t)?.label ?? "";
  });

  function beginTool(tool: ImageEditTool) {
    if (!nodeOperations.allows("edit") || session.applying || (tool === "crop" && !dimensionsReady.value))
      return;
    if (commenting.value) comments.cancel();
    openPopover.value = undefined;
    session.begin(node.value.id, tool);
  }

  function beginRemoval() {
    if (!nodeOperations.allows("derive") || !removalMode.value) return;
    if (removalMode.value === "mask") {
      openPopover.value = undefined;
      removal.begin(node.value.id);
    } else {
      openPopover.value = "prompt-removal";
    }
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
      if (removal.active?.nodeId === previous) removal.cancel();
      if (comments.activeNodeId === previous) comments.cancel();
    },
  );
  onBeforeUnmount(() => {
    if (session.active?.nodeId === node.value.id) session.cancel();
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
    removalSizeValid,
    removalMode,
    removalReason,
    beginTool,
    beginRemoval,
    setPopover,
    beginAngle,
    beginComment,
    removeBackground,
  };
}
