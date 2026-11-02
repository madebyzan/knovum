<script setup lang="ts">
import { BookOpen } from "@lucide/vue";
import { shallowRef } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import SpacePdfReader from "./SpacePdfReader.vue";

defineOptions({ inheritAttrs: false });
defineProps<{
  title: string;
  path: string;
  page?: number | undefined;
  passage?: string | undefined;
  size: "sm" | "md";
}>();
const open = shallowRef(false);
</script>

<template>
  <UiButton v-bind="$attrs" variant="secondary" :size="size" class="gap-1.5" @click="open = true">
    <BookOpen class="size-3.5" />
    <span>{{
      page ? $t("spaces.inspector.pdfReader.openAtPage", { page }) : $t("spaces.inspector.pdfReader.open")
    }}</span>
  </UiButton>
  <SpacePdfReader
    v-if="open"
    v-model:open="open"
    :title="title"
    :path="path"
    :page="page"
    :passage="passage"
  />
</template>
