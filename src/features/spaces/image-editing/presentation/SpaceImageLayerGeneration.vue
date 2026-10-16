<script setup lang="ts">
import { Layers3 } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { CANVAS_ACTION_BUTTON_CLASSES } from "../../canvas/presentation/canvas-action-classes";
import {
  NODE_ACTION_LABEL_CLASS,
  nodeActionLabelState,
} from "../../canvas/presentation/selection/node-action-label";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { splitImageLayers } from "../../knowledge-derivations/image-layers";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload>; compact?: boolean }>();
const nodeOperations = useNodeOperationGate();
const { t } = useI18n();
const busy = shallowRef(false);
const disabled = computed(
  () => busy.value || !props.node.data.sourceRevision || !nodeOperations.allows("derive"),
);

async function split() {
  if (disabled.value) return;
  busy.value = true;
  try {
    await splitImageLayers(props.node);
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <ToolbarButton as-child>
    <button
      type="button"
      :class="CANVAS_ACTION_BUTTON_CLASSES"
      :disabled="disabled"
      :aria-label="t('spaces.media.imageEditor.layers.title')"
      :title="compact ? t('spaces.media.imageEditor.layers.title') : undefined"
      @click="split"
    >
      <Layers3 class="size-4.5" :stroke-width="1.6" />
      <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(Boolean(compact))]">{{
        t("spaces.media.imageEditor.layers.title")
      }}</span>
    </button>
  </ToolbarButton>
</template>
