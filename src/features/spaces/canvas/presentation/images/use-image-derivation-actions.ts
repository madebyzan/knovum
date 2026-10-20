import { computed, shallowRef, type Ref } from "vue";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import { canRunAction } from "../../../../../shared/contracts/action-availability";
import type { ImageExecutionTarget } from "../../../contracts/generated/ImageExecutionTarget";
import type { ImageOperation } from "../../../contracts/generated/ImageOperation";
import { imageEditOf, rotatedImageSize } from "../../../image-editing/image-edit";
import { deriveImage } from "../../../knowledge-derivations/image-operation";
import { useImageOperationCapabilities } from "../../../knowledge-derivations/use-image-operation-capabilities";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { useNodeOperationGate } from "../../use-node-operation-gate";
import { imageActionAvailability, upscaleAvailability } from "./image-action-availability";
import { imageActionHelp } from "./image-action-help";
import { useImageSourceDimensionsStore } from "./image-source-dimensions-store";

interface ImageDerivationActionsInput {
  node: Readonly<Ref<SpaceCanvasNode<SpaceImagePayload>>>;
  mode: "upscale" | "angle";
  open: Readonly<Ref<boolean>>;
  close: () => void;
}

export function useImageDerivationActions({ node, mode, open, close }: ImageDerivationActionsInput) {
  const router = useRouter();
  const imageDimensions = useImageSourceDimensionsStore();
  const nodeOperations = useNodeOperationGate();
  const capabilities = useImageOperationCapabilities(open);
  const busy = shallowRef(false);
  const selectedExecution = shallowRef<ImageExecutionTarget>();
  const targetOptions = computed(
    () => capabilities.data.value?.targets.filter((option) => option.capability === "imageUpscaling") ?? [],
  );
  const executionOption = computed(() => {
    const selected = selectedExecution.value;
    return selected
      ? targetOptions.value.find((option) => JSON.stringify(option.execution) === JSON.stringify(selected))
      : targetOptions.value.find((option) => option.selected);
  });
  const dimensions = computed(() => {
    const loaded = imageDimensions.forNode(node.value.id, node.value.data.sourceRevision);
    const width = node.value.data.intrinsicWidth ?? loaded?.width;
    const height = node.value.data.intrinsicHeight ?? loaded?.height;
    if (!width || !height) return undefined;
    const edit = imageEditOf(node.value.data);
    return rotatedImageSize(width * edit.crop.width, height * edit.crop.height, edit.rotation);
  });
  const availability = computed(() =>
    imageActionAvailability({
      sourceReady: Boolean(node.value.data.sourceRevision),
      editing: !nodeOperations.allows("derive"),
      capability: capabilities.isFetching.value
        ? undefined
        : mode === "upscale"
          ? executionOption.value?.availability
          : capabilities.data.value?.multiAngle,
      failed: capabilities.isError.value && !capabilities.isFetching.value,
      busy: busy.value,
    }),
  );
  const help = computed(() => imageActionHelp(availability.value));

  function targetDimensions(longEdge: number) {
    const source = dimensions.value;
    if (!source) return undefined;
    const scale = longEdge / Math.max(source.width, source.height);
    return `${Math.round(source.width * scale)} × ${Math.round(source.height * scale)}`;
  }

  function optionAvailability(size: number) {
    const target = executionOption.value;
    return target
      ? upscaleAvailability(availability.value, dimensions.value, size, target)
      : availability.value;
  }

  function openAiSettings() {
    close();
    void router.push({ name: "settings", query: { pane: "aiProviders" } });
  }

  async function run(operation: ImageOperation) {
    if (!nodeOperations.allows("derive") || !canRunAction(availability.value)) return;
    if (operation.kind === "upscale" && !canRunAction(optionAvailability(operation.targetLongEdge))) return;
    close();
    busy.value = true;
    try {
      await deriveImage(node.value, operation, {
        execution: mode === "upscale" ? executionOption.value?.execution : undefined,
      });
    } catch (error) {
      toast.error(getErrorMessage(error));
    } finally {
      busy.value = false;
    }
  }

  return {
    busy,
    targetOptions,
    executionOption,
    selectExecution: (execution: ImageExecutionTarget) => (selectedExecution.value = execution),
    availability,
    help,
    targetDimensions,
    optionAvailability,
    openAiSettings,
    run,
  };
}
