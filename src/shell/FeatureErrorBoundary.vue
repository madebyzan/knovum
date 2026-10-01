<script setup lang="ts">
import { AlertTriangle, RotateCcw } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { ErrorCodes, onErrorCaptured, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { formatErrorDiagnostic } from "../foundation/errors/error-format";
import UiButton from "../shared/ui/UiButton.vue";

const props = defineProps<{ resetKey: string }>();
const failed = shallowRef(false);
const { t } = useI18n();

// Vue supplies descriptions in development and reference URLs in production.
const renderingErrors = new Set(
  import.meta.env.PROD
    ? [ErrorCodes.SETUP_FUNCTION, ErrorCodes.RENDER_FUNCTION, ErrorCodes.ASYNC_COMPONENT_LOADER].map(
        (code) => `https://vuejs.org/error-reference/#runtime-${code}`,
      )
    : ["setup function", "render function", "async component loader"],
);

watch(() => props.resetKey, retry);

onErrorCaptured((caught, _instance, info) => {
  if (!renderingErrors.has(info)) return;
  if (!failed.value) {
    failed.value = true;
    void logError(`Content rendering failed (${info}): ${formatErrorDiagnostic(caught)}`);
  }
  return false;
});

function retry() {
  failed.value = false;
}
</script>

<template>
  <slot v-if="!failed" />
  <div
    v-else
    data-testid="content-render-error"
    role="alert"
    class="grid h-full place-items-center bg-surface-canvas p-6"
  >
    <section class="flex max-w-sm flex-col items-center text-center">
      <AlertTriangle aria-hidden="true" class="mb-3 size-5 text-fg-secondary" />
      <h2 class="m-0 text-label font-medium text-fg-primary">{{ t("errors.feature.title") }}</h2>
      <p class="mt-1.5 text-body text-fg-secondary">{{ t("errors.feature.description") }}</p>
      <UiButton class="mt-4" size="sm" @click="retry">
        <RotateCcw aria-hidden="true" class="size-3.5" /> {{ t("errors.feature.reopen") }}
      </UiButton>
    </section>
  </div>
</template>
