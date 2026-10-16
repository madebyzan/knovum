<script setup lang="ts">
import { SliderRange, SliderRoot, SliderThumb, SliderTrack } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps<{
  currentMs: number;
  durationMs: number;
}>();
const emit = defineEmits<{ seek: [positionMs: number] }>();
const { t } = useI18n();

const position = computed(() => Math.min(props.currentMs, props.durationMs));

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
        class="relative h-1 w-full overflow-hidden rounded-full bg-white/25 transition-[height] duration-150 group-hover/timeline:h-1.5"
      >
        <SliderRange class="absolute h-full rounded-full bg-white" />
      </SliderTrack>
      <SliderThumb
        class="block size-2.5 rounded-full bg-white shadow-sm ring-1 ring-black/20 transition-transform duration-100 ease-standard outline-none group-hover/timeline:scale-125 focus-visible:scale-125 focus-visible:ring-2 focus-visible:ring-stroke-focus"
      />
    </SliderRoot>
  </div>
</template>
