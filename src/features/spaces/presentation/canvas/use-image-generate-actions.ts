import { computed, shallowRef, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { canRunAction } from "../../../../shared/contracts/action-availability";
import { useNodeOperationGate } from "../../canvas/node-operation-gate";
import { imageEditOf, rotatedImageSize } from "../../image-editing/image-edit";
import { useImageEditSession } from "../../image-editing/image-edit-session";
import { useImageOperationCapabilities } from "../../knowledge-derivations/image-capabilities";
import { deriveImage, type ImageOperation } from "../../knowledge-derivations/image-operation";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import { imageActionAvailability, upscaleAvailability } from "./image-action-availability";
import { imageActionHelp } from "./image-action-help";

export function useImageGenerateActions(
  node: Readonly<Ref<SpaceCanvasNode<SpaceImagePayload>>>,
  mode: "upscale" | "angle",
  open: Readonly<Ref<boolean>>,
  close: () => void,
) {
  const { t } = useI18n();
  const router = useRouter();
  const store = useSpaceStore();
  const session = useImageEditSession();
  const nodeOperations = useNodeOperationGate();
  const capabilities = useImageOperationCapabilities(open);
  const busy = shallowRef(false);
  const dimensions = computed(() => {
    const loaded = session.dimensionsFor(node.value.id);
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
          ? capabilities.data.value?.upscale
          : capabilities.data.value?.multiAngle,
      failed: capabilities.isError.value && !capabilities.isFetching.value,
      busy: busy.value,
    }),
  );
  const help = computed(() => imageActionHelp(availability.value, t));

  function targetDimensions(longEdge: number) {
    const source = dimensions.value;
    if (!source) return undefined;
    const scale = longEdge / Math.max(source.width, source.height);
    return `${Math.round(source.width * scale)} × ${Math.round(source.height * scale)}`;
  }

  function optionAvailability(size: number) {
    return upscaleAvailability(availability.value, dimensions.value, size);
  }

  function openAiSettings() {
    close();
    void router.push({ name: "settings", query: { pane: "aiModels" } });
  }

  async function run(operation: ImageOperation) {
    if (!nodeOperations.allows("derive") || !canRunAction(availability.value)) return;
    if (operation.kind === "upscale" && !canRunAction(optionAvailability(operation.targetLongEdge))) return;
    close();
    busy.value = true;
    try {
      await deriveImage(store, node.value, operation);
    } catch (error) {
      toast.error(String(error));
    } finally {
      busy.value = false;
    }
  }

  return {
    busy,
    availability,
    help,
    targetDimensions,
    optionAvailability,
    openAiSettings,
    run,
  };
}
