<script setup lang="ts">
import { FlipHorizontal2, FlipVertical2, RotateCw } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import type { DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import { cn } from "../../../../shared/ui/cn";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { CANVAS_ACTION_BAR_CLASSES } from "../../canvas/presentation/canvas-action-classes";
import type { ImageEdit } from "../image-edit";
import { nextQuarterTurn } from "../image-edit";

const props = defineProps<{ edit: DeepReadonly<ImageEdit>; busy: boolean }>();
const emit = defineEmits<{ update: [patch: Partial<ImageEdit>]; cancel: []; apply: [] }>();
const { t } = useI18n();

function rotate() {
  emit("update", { rotation: nextQuarterTurn(props.edit.rotation) });
}

function setRotation(event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(value) && value >= 0 && value < 360) emit("update", { rotation: value });
}
</script>

<template>
  <ToolbarRoot
    :class="cn(CANVAS_ACTION_BAR_CLASSES, 'gap-1 text-fg-primary')"
    :aria-label="t('spaces.media.transform.title')"
  >
    <span class="px-2 text-label font-medium">{{ t("spaces.media.transform.title") }}</span>
    <label
      class="flex h-full items-center gap-0.5 border-l border-stroke-divider px-2 text-label tabular-nums"
    >
      <input
        type="number"
        :aria-label="t('spaces.media.transform.rotationDegrees')"
        :min="0"
        :max="359"
        :value="edit.rotation"
        class="w-10 rounded-md bg-transparent text-right text-fg-primary outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus"
        :disabled="busy"
        @change="setRotation"
      />°
    </label>
    <div class="flex h-full items-center border-l border-stroke-divider px-1">
      <ToolbarButton as-child>
        <UiIconButton :label="t('spaces.media.transform.rotateClockwise')" :disabled="busy" @click="rotate">
          <RotateCw class="size-4.5" :stroke-width="1.6" />
        </UiIconButton>
      </ToolbarButton>
      <ToolbarButton as-child>
        <UiIconButton
          :label="t('spaces.media.transform.flipHorizontal')"
          tooltip
          :active="edit.flipHorizontal"
          :disabled="busy"
          @click="emit('update', { flipHorizontal: !edit.flipHorizontal })"
        >
          <FlipHorizontal2 class="size-4.5" :stroke-width="1.6" />
        </UiIconButton>
      </ToolbarButton>
      <ToolbarButton as-child>
        <UiIconButton
          :label="t('spaces.media.transform.flipVertical')"
          tooltip
          :active="edit.flipVertical"
          :disabled="busy"
          @click="emit('update', { flipVertical: !edit.flipVertical })"
        >
          <FlipVertical2 class="size-4.5" :stroke-width="1.6" />
        </UiIconButton>
      </ToolbarButton>
    </div>
    <div class="flex h-full items-center gap-1 border-l border-stroke-divider px-1">
      <UiButton variant="ghost" class="shadow-none" :disabled="busy" @click="emit('cancel')">{{
        t("common.cancel")
      }}</UiButton>
      <UiButton variant="primary" :disabled="busy" @click="emit('apply')">{{ t("common.apply") }}</UiButton>
    </div>
  </ToolbarRoot>
</template>
