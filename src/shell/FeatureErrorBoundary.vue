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
  <main v-else class="grid h-full place-items-center bg-canvas p-8">
    <section
      class="w-full max-w-(--knovum-error-max-width) rounded-dialog border border-border-strong bg-raised p-5"
    >
      <AlertTriangle class="mb-3 size-(--knovum-icon-size-lg) text-danger" />
      <h1 class="m-0 text-title-sm font-semibold">{{ t("errors.feature.title") }}</h1>
      <p class="mt-1.5 text-body text-foreground-secondary">{{ t("errors.feature.description") }}</p>
      <UiButton class="mt-4" @click="retry"
        ><RotateCcw class="size-(--knovum-icon-size-sm)" /> {{ t("common.retry") }}</UiButton
      >
    </section>
  </main>
</template>
