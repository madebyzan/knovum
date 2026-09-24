<script setup lang="ts">
import { FlipHorizontal2, FlipVertical2, RotateCw } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { ImageEdit } from "../../image-editing/image-edit";
import { nextQuarterTurn } from "../../image-editing/image-edit";

const props = defineProps<{ edit: ImageEdit; busy: boolean }>();
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
    class="flex h-(--knovum-control-height-lg) w-max items-center gap-1 rounded-panel border border-border-strong bg-overlay p-px text-foreground shadow-control"
    :aria-label="t('spaces.media.transformImage')"
  >
    <span class="px-2 text-label font-medium">{{ t("spaces.media.transformImage") }}</span>
    <label class="flex h-full items-center gap-0.5 border-l border-divider px-2 text-label tabular-nums">
      <input
        type="number"
        :aria-label="t('spaces.media.rotationDegrees')"
        :min="0"
        :max="359"
        :value="edit.rotation"
        class="w-10 rounded-control bg-transparent text-right text-foreground outline-none focus-visible:ring-2 focus-visible:ring-focus-ring"
        :disabled="busy"
        @change="setRotation"
      />°
    </label>
    <div class="flex h-full items-center border-l border-divider px-1">
      <ToolbarButton as-child>
        <UiIconButton :label="t('spaces.media.rotateClockwise')" :disabled="busy" @click="rotate">
          <RotateCw class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </ToolbarButton>
      <ToolbarButton as-child>
        <UiIconButton
          :label="t('spaces.media.flipHorizontal')"
          :active="edit.flipHorizontal"
          :disabled="busy"
          @click="emit('update', { flipHorizontal: !edit.flipHorizontal })"
        >
          <FlipHorizontal2 class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </ToolbarButton>
      <ToolbarButton as-child>
        <UiIconButton
          :label="t('spaces.media.flipVertical')"
          :active="edit.flipVertical"
          :disabled="busy"
          @click="emit('update', { flipVertical: !edit.flipVertical })"
        >
          <FlipVertical2 class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </ToolbarButton>
    </div>
    <div class="flex h-full items-center gap-1 border-l border-divider px-1">
      <UiButton variant="ghost" size="sm" class="shadow-none" :disabled="busy" @click="emit('cancel')">{{
        t("common.cancel")
      }}</UiButton>
      <UiButton variant="primary" size="sm" :disabled="busy" @click="emit('apply')">{{
        t("common.apply")
      }}</UiButton>
    </div>
  </ToolbarRoot>
</template>
