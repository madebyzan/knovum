<script setup lang="ts">
import { useI18n } from "vue-i18n";

defineProps<{ titles: readonly string[] }>();
const emit = defineEmits<{ back: [] }>();
const { t } = useI18n();
</script>

<template>
  <div
    class="absolute top-3 left-1/2 z-30 flex -translate-x-1/2 items-center gap-2 rounded-md border border-stroke-default bg-surface-muted/95 px-2 py-1 shadow-popover backdrop-blur"
  >
    <button
      type="button"
      class="rounded-md px-1.5 py-0.5 text-caption-md text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary"
      @click="emit('back')"
    >
      {{ t("common.back") }}
    </button>
    <span class="bg-border-default h-3 w-px" />
    <span class="max-w-72 truncate text-caption-md font-medium text-fg-primary">
      {{ titles.join(" / ") }}
    </span>
  </div>
</template>
