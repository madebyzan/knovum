<script setup lang="ts">
import SpaceVideoTimeField from "./SpaceVideoTimeField.vue";
const props = defineProps<{ fadeInMs: number; fadeOutMs: number; durationMs: number }>();
defineEmits<{ change: [value: { fadeInMs?: number; fadeOutMs?: number }] }>();
</script>
<template>
  <section class="grid grid-cols-2 gap-2">
    <SpaceVideoTimeField
      :label="$t('spaces.media.videoEditor.fades.in')"
      :value="fadeInMs"
      :max="durationMs - fadeOutMs"
      @change="$emit('change', { fadeInMs: $event })"
    />
    <SpaceVideoTimeField
      :label="$t('spaces.media.videoEditor.fades.out')"
      :value="fadeOutMs"
      :max="durationMs - fadeInMs"
      @change="$emit('change', { fadeOutMs: $event })"
    />
  </section>
</template>
