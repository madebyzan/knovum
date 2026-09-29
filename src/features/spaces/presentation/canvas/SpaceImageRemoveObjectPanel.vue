<script setup lang="ts">
import { Undo2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import SpaceImageRangeSlider from "./SpaceImageRangeSlider.vue";

const props = defineProps<{
  diameter: number;
  canUndo: boolean;
  canGenerate: boolean;
  submitting: boolean;
  hint?: string;
}>();
const emit = defineEmits<{
  updateDiameter: [diameter: number];
  undo: [];
  cancel: [];
  generate: [];
}>();
const { t } = useI18n();
const brushSize = computed({
  get: () => Math.round(props.diameter * 1000),
  set: (value: number) => emit("updateDiameter", value / 1000),
});
</script>

<template>
  <section
    class="w-56 space-y-3 text-fg-primary"
    :aria-label="t('spaces.media.imageEditor.removeObject.tool')"
  >
    <header class="flex items-center justify-between px-1">
      <h3 class="text-title-sm font-semibold">{{ t("spaces.media.imageEditor.removeObject.tool") }}</h3>
      <UiButton
        variant="ghost"
        size="sm"
        class="shadow-none"
        :disabled="!canUndo || submitting"
        :aria-label="t('spaces.media.imageEditor.removeObject.undo')"
        @click="emit('undo')"
      >
        <Undo2 class="size-3.5" />
      </UiButton>
    </header>
    <p class="px-1 text-caption text-fg-secondary">
      {{ t("spaces.media.imageEditor.removeObject.instruction") }}
    </p>
    <SpaceImageRangeSlider
      minimal
      :label="t('spaces.media.imageEditor.removeObject.brushSize')"
      :value="brushSize"
      :min="5"
      :max="150"
      :disabled="submitting"
      @update="brushSize = $event"
    />
    <p v-if="hint" class="px-1 text-caption text-fg-secondary">{{ hint }}</p>
    <footer class="flex gap-2 border-t border-stroke-divider pt-2">
      <UiButton
        variant="secondary"
        size="sm"
        class="min-w-0 flex-1 border border-stroke-default bg-surface-raised"
        :disabled="submitting"
        @click="emit('cancel')"
        >{{ t("common.cancel") }}</UiButton
      >
      <UiButton
        variant="primary"
        size="sm"
        class="min-w-0 flex-1 bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90"
        :disabled="!canGenerate || submitting"
        @click="emit('generate')"
        >{{ t("spaces.media.imageEditor.removeObject.generate") }}</UiButton
      >
    </footer>
  </section>
</template>
