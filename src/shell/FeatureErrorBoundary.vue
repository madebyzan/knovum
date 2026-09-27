<script setup lang="ts">
import { AlertTriangle, RotateCcw } from "@lucide/vue";
import { nextTick, onErrorCaptured, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../shared/ui/UiButton.vue";
import { handleBoundaryError } from "./error-handling";

const props = defineProps<{ resetKey: string }>();
const failure = ref<Error>();
const { t } = useI18n();

watch(
  () => props.resetKey,
  () => {
    failure.value = undefined;
  },
);

onErrorCaptured((caught) => {
  failure.value = handleBoundaryError(caught, "feature-error-boundary");
  return false;
});

async function retry() {
  failure.value = undefined;
  await nextTick();
}
</script>

<template>
  <slot v-if="!failure" />
  <main v-else class="grid h-full place-items-center bg-surface-canvas p-8">
    <section class="w-full max-w-130 rounded-xl border border-stroke-strong bg-surface-raised p-5">
      <AlertTriangle class="mb-3 size-5 text-fg-danger" />
      <h1 class="m-0 text-title-sm font-semibold">{{ t("errors.feature.title") }}</h1>
      <p class="mt-1.5 text-body text-fg-secondary">{{ t("errors.feature.description") }}</p>
      <UiButton class="mt-4" @click="retry"><RotateCcw class="size-3.5" /> {{ t("common.retry") }}</UiButton>
    </section>
  </main>
</template>
