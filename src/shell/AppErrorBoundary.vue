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
  <main v-else class="grid h-full place-items-center bg-chrome p-8">
    <section
      class="w-full max-w-(--knovum-error-max-width) rounded-dialog border border-border-strong bg-raised p-5 shadow-dialog"
    >
      <AlertTriangle class="mb-3 size-(--knovum-icon-size-lg) text-danger" />
      <h1 class="m-0 text-title-sm font-semibold">{{ t("errors.title") }}</h1>
      <p class="mt-1.5 text-body text-foreground-secondary">{{ t("errors.description") }}</p>
      <pre
        class="mt-4 max-h-40 overflow-auto rounded-control bg-chrome p-3 font-mono text-micro text-foreground-secondary select-text"
        >{{ failure.stack ?? failure.message }}</pre>
      <UiButton class="mt-4" @click="reload"
        ><RotateCcw class="size-(--knovum-icon-size-sm)" /> {{ t("common.reload") }}</UiButton
      >
    </section>
  </main>
</template>
