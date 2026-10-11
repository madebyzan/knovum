<script setup lang="ts">
import { ScanLine } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import { canRunAction, showsAction } from "../../../../shared/contracts/action-availability";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
import type { ImageActionAvailability } from "./image-action-availability";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "./node-action-label";

defineProps<{ compact: boolean; availability: ImageActionAvailability; reason: string }>();
const emit = defineEmits<{ begin: [] }>();
const { t } = useI18n();
const REMOVE_BACKGROUND_LABEL_KEY = "spaces.creation.image.removeBackground";
</script>

<template>
  <UiTooltip
    v-if="showsAction(availability)"
    :text="canRunAction(availability) ? t(REMOVE_BACKGROUND_LABEL_KEY) : reason"
    :disabled="canRunAction(availability) && !compact"
    side="top"
  >
    <ToolbarButton as-child>
      <button
        type="button"
        :class="CANVAS_ACTION_BUTTON_CLASSES"
        :aria-disabled="!canRunAction(availability)"
        :aria-label="t(REMOVE_BACKGROUND_LABEL_KEY)"
        @click="emit('begin')"
      >
        <ScanLine class="size-4.5" :stroke-width="1.6" />
        <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(compact)]">{{
          t(REMOVE_BACKGROUND_LABEL_KEY)
        }}</span>
      </button>
    </ToolbarButton>
  </UiTooltip>
</template>
