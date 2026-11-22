<script setup lang="ts">
import { ChevronDown, ScanText } from "@lucide/vue";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

defineProps<{
  page: number;
  passages: readonly string[];
}>();

const { t } = useI18n();
const open = shallowRef(false);
</script>

<template>
  <div
    :data-page-number="page"
    class="max-h-48 shrink-0 overflow-y-auto border-t border-stroke-divider/70 bg-surface-raised px-4 py-2"
  >
    <button
      type="button"
      data-reader-ignore-selection
      class="flex w-full items-center gap-1.5 text-left text-caption-sm font-medium text-fg-secondary select-none hover:text-fg-primary"
      @click="open = !open"
    >
      <ScanText class="size-3.5 text-fg-tertiary" :stroke-width="1.6" />
      <span class="min-w-0 flex-1 truncate">{{ t("spaces.inspector.recognizedText") }}</span>
      <ChevronDown
        class="size-3 shrink-0 text-fg-tertiary transition-transform duration-150 ease-standard"
        :class="open && 'rotate-180'"
        :stroke-width="1.6"
      />
    </button>
    <div v-if="open" class="mt-2.5 space-y-2.5 pb-1 text-body leading-relaxed text-fg-primary">
      <p v-for="(text, index) in passages" :key="index" class="whitespace-pre-wrap">
        {{ text }}
      </p>
    </div>
  </div>
</template>
