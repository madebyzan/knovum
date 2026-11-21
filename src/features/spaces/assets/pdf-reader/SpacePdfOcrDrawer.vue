<script setup lang="ts">
import { ScanText } from "@lucide/vue";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

defineProps<{
  page: number;
  passages: readonly string[];
}>();

const { t } = useI18n();
const open = shallowRef(true);
</script>

<template>
  <div
    :data-page-number="page"
    class="max-h-44 shrink-0 overflow-y-auto border-t border-stroke-divider bg-surface-panel px-3.5 py-2.5"
  >
    <button
      type="button"
      data-reader-ignore-selection
      class="flex w-full items-center gap-1.5 text-left text-caption-sm font-medium text-fg-secondary select-none"
      @click="open = !open"
    >
      <ScanText class="size-3.5 text-fg-tertiary" />
      <span>{{ t("spaces.inspector.recognizedText") }}</span>
    </button>
    <div v-if="open" class="mt-2 space-y-2">
      <p
        v-for="(text, index) in passages"
        :key="index"
        class="border-l-2 border-surface-accent/70 py-0.5 pl-3 text-body leading-relaxed whitespace-pre-wrap text-fg-primary"
      >
        {{ text }}
      </p>
    </div>
  </div>
</template>
