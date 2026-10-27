<script setup lang="ts">
import type { EditorSaveStatus } from "../use-editor-autosave";

defineProps<{ status: EditorSaveStatus }>();
</script>

<template>
  <span class="text-caption-sm font-medium text-fg-tertiary" aria-live="polite">
    {{
      status === "saving"
        ? $t("common.saving")
        : status === "unsaved"
          ? $t("common.unsaved")
          : $t("common.saved")
    }}
  </span>
</template>
