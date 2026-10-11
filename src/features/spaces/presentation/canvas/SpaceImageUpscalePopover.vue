<script setup lang="ts">
import { Maximize2 } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { toRef } from "vue";
import { useI18n } from "vue-i18n";

import { canRunAction } from "../../../../shared/contracts/action-availability";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
import { imageActionHelp } from "./image-action-help";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "./node-action-label";
import SpaceImageExecutionSelect from "./SpaceImageExecutionSelect.vue";
import { useImageDerivationActions } from "./use-image-derivation-actions";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceImagePayload>;
  compact?: boolean;
  open: boolean;
}>();
const emit = defineEmits<{ "update:open": [open: boolean] }>();
const { t } = useI18n();
const {
  busy,
  targetOptions,
  executionOption,
  selectExecution,
  help,
  targetDimensions,
  optionAvailability,
  openAiSettings,
  run,
} = useImageDerivationActions({
  node: toRef(props, "node"),
  mode: "upscale",
  open: toRef(props, "open"),
  close: () => emit("update:open", false),
});
const UPSCALE_SIZES = [2048, 4096, 8192] as const;
</script>

<template>
  <UiPopover
    :open="open"
    side="bottom"
    align="start"
    class="w-48 p-1"
    @update:open="emit('update:open', $event)"
  >
    <template #trigger>
      <ToolbarButton as-child>
        <button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :disabled="busy"
          :aria-label="t('spaces.media.imageEditor.upscale.action')"
          :title="compact ? t('spaces.media.imageEditor.upscale.action') : undefined"
        >
          <Maximize2 class="size-4.5" :stroke-width="1.6" />
          <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(Boolean(compact))]">{{
            t("spaces.media.imageEditor.upscale.action")
          }}</span>
        </button>
      </ToolbarButton>
    </template>
    <div class="border-b border-stroke-divider p-1 pb-2">
      <SpaceImageExecutionSelect
        :options="targetOptions"
        :execution="executionOption?.execution"
        @select="selectExecution"
      />
    </div>
    <UiTooltip
      v-for="size in UPSCALE_SIZES"
      :key="size"
      :text="imageActionHelp(optionAvailability(size))?.label ?? ''"
      :disabled="canRunAction(optionAvailability(size))"
      side="right"
    >
      <button
        type="button"
        class="flex h-(--knovum-control-height-lg) w-full items-center justify-between gap-2 rounded-md px-2 text-left text-label outline-none hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus aria-disabled:cursor-default aria-disabled:opacity-40"
        :aria-disabled="!canRunAction(optionAvailability(size))"
        @click="run({ kind: 'upscale', targetLongEdge: size })"
      >
        <span class="shrink-0 font-medium text-fg-primary">{{ size / 1024 }}K</span>
        <span v-if="targetDimensions(size)" class="text-fg-tertiary tabular-nums">{{
          targetDimensions(size)
        }}</span>
      </button>
    </UiTooltip>
    <div v-if="help?.actionLabel" class="flex justify-end border-t border-stroke-divider pt-1">
      <button
        type="button"
        class="px-2 py-1 text-caption text-fg-accent hover:underline"
        @click="openAiSettings"
      >
        {{ help.actionLabel }}
      </button>
    </div>
  </UiPopover>
</template>
