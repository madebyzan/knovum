<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSlider from "../../../../shared/ui/UiSlider.vue";

const props = defineProps<{
  currentMs: number;
  durationMs: number;
}>();
const emit = defineEmits<{ seek: [positionMs: number] }>();
const { t } = useI18n();
const position = computed({
  get: () => Math.min(props.currentMs, props.durationMs),
  set: (value: number) => emit("seek", value),
});
</script>

<template>
  <div class="min-w-0 flex-1">
    <UiSlider
      v-model="position"
      :label="t('spaces.media.player.timeline')"
      :min="0"
      :max="Math.max(durationMs, 1)"
      :step="100"
    />
  </div>
</template>
