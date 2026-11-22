<script setup lang="ts">
import { Check, Copy, Search } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

defineProps<{
  address: string;
  formulaText: string;
  hasSelection: boolean;
  copied: boolean;
  findOpen: boolean;
}>();

const emit = defineEmits<{ copy: []; toggleFind: [] }>();
const { t } = useI18n();
</script>

<template>
  <div
    class="flex h-8.5 shrink-0 items-center gap-2 border-b border-stroke-divider/70 bg-surface-raised px-3"
  >
    <span
      class="min-w-12 shrink-0 rounded-sm bg-surface-control px-1.5 py-0.5 text-center font-mono text-caption-sm font-medium text-fg-secondary"
    >
      {{ address }}
    </span>
    <span class="font-mono text-caption-sm font-medium text-fg-tertiary">fx</span>
    <span class="min-w-0 flex-1 truncate font-mono text-caption-sm text-fg-primary" :title="formulaText">
      {{ formulaText }}
    </span>
    <div class="flex shrink-0 items-center gap-0.5">
      <UiIconButton
        v-if="hasSelection"
        :label="copied ? t('common.copied') : t('common.copy')"
        size="sm"
        tooltip
        @click="emit('copy')"
      >
        <Check v-if="copied" class="size-3.5 text-fg-accent" />
        <Copy v-else class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.inspector.reader.searchPlaceholder')"
        size="sm"
        tooltip
        :class="findOpen && 'bg-surface-control text-fg-primary'"
        @click="emit('toggleFind')"
      >
        <Search class="size-3.5" />
      </UiIconButton>
    </div>
  </div>
</template>
