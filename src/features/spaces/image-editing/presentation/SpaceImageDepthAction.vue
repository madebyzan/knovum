<script setup lang="ts">
import { Layers } from "@lucide/vue";
import { computed } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { availableWhen } from "../../../../shared/contracts/action-availability";
import type { ImageActionAvailability } from "../../canvas/presentation/images/image-action-availability";
import SpaceImageDirectAction from "../../canvas/presentation/images/SpaceImageDirectAction.vue";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import type { ImageExecutionTarget } from "../../contracts/generated/ImageExecutionTarget";
import { deriveImage } from "../../knowledge-derivations/image-operation";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const gate = useNodeOperationGate();
const availability = computed<ImageActionAvailability>(() =>
  availableWhen(gate.allows("derive") && Boolean(props.node.data.sourceRevision), "image_preparing"),
);

function depth(execution?: ImageExecutionTarget) {
  if (!gate.allows("derive") || !props.node.data.sourceRevision) return;
  void deriveImage(props.node, { kind: "generate-depth" }, { execution }).catch((cause: unknown) =>
    toast.error(getErrorMessage(cause)),
  );
}
</script>

<template>
  <SpaceImageDirectAction
    capability="imageDepth"
    :availability="availability"
    reason=""
    :icon="Layers"
    label-key="spaces.media.imageEditor.depth.action"
    menu-item
    @run="depth"
  />
</template>
