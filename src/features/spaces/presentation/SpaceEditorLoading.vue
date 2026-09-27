<script setup lang="ts">
import { FileText } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiSpinner from "../../../shared/ui/UiSpinner.vue";

const { t } = useI18n();
const toolbarPlaceholders = ["w-8", "w-8", "w-12", "w-8", "w-8", "w-8", "w-16"];
const paragraphPlaceholders = ["w-full", "w-11/12", "w-4/5", "w-full", "w-2/3"];
</script>

<template>
  <section
    class="absolute inset-0 z-30 flex min-h-0 flex-col bg-surface-canvas"
    aria-live="polite"
    aria-busy="true"
  >
    <header
      class="flex h-(--knovum-editor-toolbar-height) shrink-0 items-center gap-2 border-b border-stroke-divider bg-surface-chrome px-3"
    >
      <FileText class="size-3.5 text-fg-secondary" :stroke-width="1.55" />
      <span class="text-caption font-medium text-fg-secondary">{{ t("spaces.nodeTypes.markdown") }}</span>
      <div class="flex-1" />
      <div class="flex items-center gap-2 text-caption text-fg-secondary">
        <UiSpinner :label="t('spaces.editor.loading')" size="sm" tone="accent" />
        <span>{{ t("spaces.editor.loading") }}</span>
      </div>
    </header>
    <div
      class="flex h-(--knovum-editor-toolbar-height) shrink-0 items-center gap-2 border-b border-stroke-divider bg-surface-chrome/80 px-3"
    >
      <span
        v-for="(placeholder, index) in toolbarPlaceholders"
        :key="index"
        class="h-5 rounded-md bg-surface-skeleton"
        :class="placeholder"
      />
    </div>
    <div class="min-h-0 flex-1 overflow-hidden">
      <article class="mx-auto w-full max-w-3xl px-10 py-12 sm:px-14">
        <div class="h-7 w-3/5 rounded-md bg-surface-skeleton" />
        <p class="mt-3 text-caption text-fg-secondary">{{ t("spaces.editor.loadingHint") }}</p>
        <div class="mt-10 space-y-3">
          <div
            v-for="(placeholder, index) in paragraphPlaceholders"
            :key="index"
            class="h-3 rounded-md bg-surface-skeleton"
            :class="placeholder"
          />
        </div>
        <div class="mt-9 h-5 w-1/3 rounded-md bg-surface-skeleton" />
        <div class="mt-5 space-y-3">
          <div class="h-3 w-full rounded-md bg-surface-skeleton" />
          <div class="h-3 w-5/6 rounded-md bg-surface-skeleton" />
          <div class="h-3 w-3/4 rounded-md bg-surface-skeleton" />
        </div>
      </article>
    </div>
  </section>
</template>
