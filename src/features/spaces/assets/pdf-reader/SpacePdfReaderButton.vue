<script setup lang="ts">
import { BookOpen } from "@lucide/vue";

import UiButton from "../../../../shared/ui/UiButton.vue";

defineOptions({ inheritAttrs: false });
defineProps<{
  page?: number | undefined;
  size: "sm" | "md";
}>();
const emit = defineEmits<{ open: [] }>();
</script>

<template>
  <UiButton v-bind="$attrs" variant="secondary" :size="size" class="gap-1.5" @click="emit('open')">
    <BookOpen class="size-3.5" />
    <span>{{
      page ? $t("spaces.inspector.pdfReader.openAtPage", { page }) : $t("spaces.inspector.pdfReader.open")
    }}</span>
  </UiButton>
</template>
