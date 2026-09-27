<script setup lang="ts">
import { AlertTriangle, RotateCcw } from "@lucide/vue";
import { onErrorCaptured, ref } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../shared/ui/UiButton.vue";
import { handleBoundaryError } from "./error-handling";

const failure = ref<Error>();
const { t } = useI18n();

onErrorCaptured((caught) => {
  failure.value = handleBoundaryError(caught);
  return false;
});

function reload() {
  globalThis.location.reload();
}
</script>

<template>
  <slot v-if="!failure" />
  <main v-else class="grid h-full place-items-center bg-surface-chrome p-8">
    <section
      class="w-full max-w-130 rounded-xl border border-stroke-strong bg-surface-raised p-5 shadow-dialog"
    >
      <AlertTriangle class="mb-3 size-5 text-fg-danger" />
      <h1 class="m-0 text-title-sm font-semibold">{{ t("errors.title") }}</h1>
      <p class="mt-1.5 text-body text-fg-secondary">{{ t("errors.description") }}</p>
      <pre
        class="mt-4 max-h-40 overflow-auto rounded-md bg-surface-chrome p-3 font-mono text-caption-sm text-fg-secondary select-text"
        >{{ failure.stack ?? failure.message }}</pre>
      <UiButton class="mt-4" @click="reload"
        ><RotateCcw class="size-3.5" /> {{ t("common.reload") }}</UiButton
      >
    </section>
  </main>
</template>
