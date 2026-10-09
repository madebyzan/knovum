<script setup lang="ts">
import { SliderRange, SliderRoot, SliderThumb, SliderTrack } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

defineProps<{ start: number; end: number; durationMs: number; disabled: boolean }>();
const emit = defineEmits<{ change: [start: number, end: number] }>();
const { t } = useI18n();
const labels = computed(() => [t("spaces.speech.reference.start"), t("spaces.speech.reference.end")]);
function change(value: number[] | undefined) {
  if (value?.length === 2) emit("change", value[0]! / 1000, value[1]! / 1000);
}
</script>

<template>
  <SliderRoot
    v-if="Number.isFinite(start) && Number.isFinite(end)"
    :model-value="[
      Math.max(0, Math.min(start * 1000, durationMs)),
      Math.max(0, Math.min(end * 1000, durationMs)),
    ]"
    :min="0"
    :max="Math.max(durationMs, 1)"
    :step="100"
    :disabled="disabled"
    :aria-label="t('spaces.speech.reference.title')"
    class="relative flex h-7 w-full touch-none items-center select-none data-disabled:opacity-40"
    @update:model-value="change"
  >
    <SliderTrack class="relative h-2 flex-1 overflow-hidden rounded-sm bg-surface-control">
      <SliderRange class="absolute h-full bg-surface-accent" />
    </SliderTrack>
    <SliderThumb
      v-for="label in labels"
      :key="label"
      :aria-label="label"
      class="block h-5 w-2.5 rounded-sm border border-stroke-strong bg-surface-raised shadow-control outline-none hover:border-stroke-contrast focus-visible:border-stroke-focus"
    />
  </SliderRoot>
</template>
