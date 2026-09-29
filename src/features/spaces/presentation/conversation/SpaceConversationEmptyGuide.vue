<script setup lang="ts">
import { FileText, Search, Waypoints } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const emit = defineEmits<{ select: [prompt: string, selection?: string] }>();
const { t } = useI18n();
const examples = computed(() => {
  const topic = t("spaces.assistant.empty.topic");
  return [
    { icon: Search, label: t("spaces.assistant.empty.search"), prompt: t("spaces.assistant.empty.searchPrompt", { topic }), selection: topic },
    { icon: FileText, label: t("spaces.assistant.empty.document"), prompt: t("spaces.assistant.empty.documentPrompt", { topic }), selection: topic },
    { icon: Waypoints, label: t("spaces.assistant.empty.compare"), prompt: t("spaces.assistant.empty.comparePrompt", { topic }), selection: topic },
  ];
});
</script>

<template>
  <div class="w-full max-w-sm">
    <h2 class="px-2.5 text-title-sm font-medium text-fg-primary">{{ t("spaces.assistant.empty.title") }}</h2>
    <div class="mt-3 space-y-1">
      <button
        v-for="example in examples"
        :key="example.label"
        type="button"
        class="flex w-full items-start gap-3 rounded-md px-2.5 py-2.5 text-left transition-colors duration-100 hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
        @click="emit('select', example.prompt, example.selection)"
      >
        <component :is="example.icon" class="mt-0.5 size-4 shrink-0 text-fg-tertiary" :stroke-width="1.6" aria-hidden="true" />
        <span class="min-w-0">
          <span class="block text-caption-sm text-fg-tertiary">{{ example.label }}</span>
          <span class="mt-0.5 block text-body text-fg-primary">{{ example.prompt }}</span>
        </span>
      </button>
    </div>
  </div>
</template>
