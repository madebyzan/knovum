<script setup lang="ts">
import { useI18n } from "vue-i18n";

defineProps<{ titles: readonly string[] }>();
const emit = defineEmits<{ back: [] }>();
const { t } = useI18n();
</script>

<template>
  <div
    class="absolute top-3 left-1/2 z-30 flex -translate-x-1/2 items-center gap-2 rounded-control border border-border bg-surface/95 px-2 py-1 shadow-popover backdrop-blur"
  >
    <button
      type="button"
      class="hover:bg-surface-hover rounded-control px-1.5 py-0.5 text-caption text-foreground-secondary hover:text-foreground"
      @click="emit('back')"
    >
      {{ t("common.back") }}
    </button>
    <span class="h-3 w-px bg-border" />
    <span class="max-w-72 truncate text-caption font-medium text-foreground">
      {{ titles.join(" / ") }}
    </span>
  </div>
</template>
