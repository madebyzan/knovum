<script setup lang="ts">
import { Undo2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import SpaceImageRangeSlider from "./SpaceImageRangeSlider.vue";

const props = defineProps<{ diameter: number; canUndo: boolean; canApply: boolean }>();
const emit = defineEmits<{
  updateDiameter: [diameter: number];
  undo: [];
  cancel: [];
  apply: [];
}>();
const { t } = useI18n();
const brushSize = computed({
  get: () => Math.round(props.diameter * 1000),
  set: (value: number) => emit("updateDiameter", value / 1000),
});
</script>

<template>
  <section class="w-56 space-y-3 text-foreground" :aria-label="t('spaces.media.eraser')">
    <header class="flex items-center justify-between px-1">
      <h3 class="text-body-lg font-semibold">{{ t("spaces.media.eraser") }}</h3>
      <UiButton variant="ghost" size="sm" class="shadow-none" :disabled="!canUndo" :title="t('spaces.media.undoEraser')" :aria-label="t('spaces.media.undoEraser')" @click="emit('undo')">
        <Undo2 class="size-(--knovum-icon-size-sm)" />
      </UiButton>
    </header>
    <SpaceImageRangeSlider minimal :label="t('spaces.media.brushSize')" :value="brushSize" :min="5" :max="150" @update="brushSize = $event" />
    <footer class="flex gap-2 border-t border-divider pt-2">
      <UiButton variant="secondary" size="sm" class="min-w-0 flex-1 border border-border bg-raised" @click="emit('cancel')">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" size="sm" class="min-w-0 flex-1 bg-foreground text-canvas hover:bg-foreground hover:opacity-90" :disabled="!canApply" @click="emit('apply')">{{ t("spaces.media.applyNow") }}</UiButton>
    </footer>
  </section>
</template>
