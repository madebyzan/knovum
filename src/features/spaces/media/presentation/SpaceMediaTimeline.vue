<script setup lang="ts">
import { SliderRange, SliderRoot, SliderThumb, SliderTrack } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const props = withDefaults(
  defineProps<{
    currentMs: number;
    durationMs: number;
    tone?: "stage" | "control" | undefined;
  }>(),
  { tone: "stage" },
);
const emit = defineEmits<{ seek: [positionMs: number] }>();
const { t } = useI18n();

const position = computed(() => Math.min(props.currentMs, props.durationMs));
const isControl = computed(() => props.tone === "control");

function onSeek(value: number[] | undefined) {
  if (value?.[0] !== undefined) emit("seek", value[0]);
}
</script>

<template>
  <div class="relative flex min-w-0 flex-1 items-center">
    <SliderRoot
      :model-value="[position]"
      :min="0"
      :max="Math.max(durationMs, 1)"
      :step="100"
      :aria-label="t('spaces.media.player.timeline')"
      class="group/timeline relative flex h-4 w-full cursor-pointer touch-none items-center select-none"
      @update:model-value="onSeek"
    >
      <SliderTrack
        class="relative h-1 w-full overflow-hidden rounded-full transition-[height] duration-150 group-hover/timeline:h-1.5"
        :class="isControl ? 'bg-stroke-strong' : 'bg-white/25'"
      >
        <SliderRange class="absolute h-full rounded-full" :class="isControl ? 'bg-fg-primary' : 'bg-white'" />
      </SliderTrack>
      <SliderThumb
        class="block size-2.5 rounded-full transition-transform duration-100 ease-standard outline-none group-hover/timeline:scale-125 focus-visible:scale-125 focus-visible:ring-2 focus-visible:ring-stroke-focus"
        :class="
          isControl
            ? 'border border-stroke-strong bg-surface-raised shadow-control'
            : 'bg-white shadow-sm ring-1 ring-black/20'
        "
      />
    </SliderRoot>
  </div>
</template>
