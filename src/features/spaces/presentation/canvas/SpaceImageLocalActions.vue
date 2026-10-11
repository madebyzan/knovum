<script setup lang="ts">
import { MousePointer2, Layers } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { deriveImage } from "../../knowledge-derivations/image-operation";
import { useImageSelectionSessionStore } from "../../knowledge-derivations/image-selection-session-store";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const { t } = useI18n();
const gate = useNodeOperationGate();
const selection = useImageSelectionSessionStore();
const disabled = computed(() => !gate.allows("derive") || !props.node.data.sourceRevision);
function depth() {
  if (!disabled.value)
    void deriveImage(props.node, { kind: "generate-depth" }).catch((cause: unknown) =>
      toast.error(getErrorMessage(cause)),
    );
}
function select() {
  if (!disabled.value) void selection.begin(props.node);
}
</script>
<template>
  <UiDropdownMenuItem :disabled="disabled" @select="select"
    ><MousePointer2 class="size-3.5" />{{ t("spaces.media.imageEditor.selection.title") }}</UiDropdownMenuItem
  >
  <UiDropdownMenuItem :disabled="disabled" @select="depth"
    ><Layers class="size-3.5" />{{ t("spaces.media.imageEditor.depth.action") }}</UiDropdownMenuItem
  >
</template>
