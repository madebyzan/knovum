<script setup lang="ts">
import { ChevronLeft, ChevronRight } from "@lucide/vue";
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

const props = defineProps<{
  currentPage: number;
  pageCount: number;
  ready: boolean;
}>();

const emit = defineEmits<{ goToPage: [page: number] }>();
const { t } = useI18n();
const pageDraft = ref(String(props.currentPage));

watch(
  () => props.currentPage,
  (page) => {
    pageDraft.value = String(page);
  },
);

function commitPage() {
  const parsed = Number.parseInt(pageDraft.value, 10);
  if (Number.isInteger(parsed) && parsed >= 1 && parsed <= props.pageCount) emit("goToPage", parsed);
  else pageDraft.value = String(props.currentPage);
}
</script>

<template>
  <div class="flex shrink-0 items-center gap-0.5">
    <UiIconButton
      :label="t('spaces.inspector.pdfReader.previousPage')"
      size="sm"
      tooltip
      :disabled="!ready || currentPage <= 1"
      @click="emit('goToPage', currentPage - 1)"
    >
      <ChevronLeft class="size-3.5" />
    </UiIconButton>
    <input
      v-model="pageDraft"
      type="text"
      inputmode="numeric"
      class="w-7 rounded-sm border border-stroke-divider bg-surface-raised py-0.5 text-center text-caption-sm text-fg-primary tabular-nums outline-none focus:border-stroke-focus"
      @keydown.enter.prevent="commitPage"
      @blur="commitPage"
    />
    <span class="px-0.5 text-caption-sm text-fg-tertiary tabular-nums">/ {{ pageCount }}</span>
    <UiIconButton
      :label="t('spaces.inspector.pdfReader.nextPage')"
      size="sm"
      tooltip
      :disabled="!ready || currentPage >= pageCount"
      @click="emit('goToPage', currentPage + 1)"
    >
      <ChevronRight class="size-3.5" />
    </UiIconButton>
  </div>
</template>
