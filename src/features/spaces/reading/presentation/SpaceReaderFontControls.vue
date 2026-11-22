<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

defineProps<{
  canDecrease: boolean;
  canIncrease: boolean;
}>();

const emit = defineEmits<{ decrease: []; increase: [] }>();
const { t } = useI18n();
</script>

<template>
  <div class="inline-flex items-center gap-0.5">
    <UiIconButton
      :label="t('spaces.inspector.reader.decreaseFontSize')"
      size="sm"
      tooltip
      :disabled="!canDecrease"
      @click="emit('decrease')"
    >
      <span class="text-caption-sm font-semibold">A−</span>
    </UiIconButton>
    <UiIconButton
      :label="t('spaces.inspector.reader.increaseFontSize')"
      size="sm"
      tooltip
      :disabled="!canIncrease"
      @click="emit('increase')"
    >
      <span class="text-caption-sm font-semibold">A+</span>
    </UiIconButton>
  </div>
</template>
