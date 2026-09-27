<script setup lang="ts">
import { AlertCircle, Clapperboard, LoaderCircle, X } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { formatByteSize } from "../../assets/format-byte-size";
import type { SpaceUrlCaptureProjection } from "../../sources/url-capture-projection";

const props = defineProps<{ capture: SpaceUrlCaptureProjection }>();
const emit = defineEmits<{ dismiss: []; retry: [] }>();
const { t } = useI18n();
const isByteTransfer = computed(
  () => props.capture.status === "preparing" || props.capture.status === "downloading",
);
const progress = computed(() => {
  if (isByteTransfer.value && props.capture.totalBytes && props.capture.totalBytes > 0) {
    return Math.min(100, Math.round((props.capture.downloadedBytes / props.capture.totalBytes) * 100));
  }
  if (
    props.capture.status === "finalizing" &&
    props.capture.processedMs !== null &&
    props.capture.durationMs &&
    props.capture.durationMs > 0
  ) {
    return Math.min(100, Math.round((props.capture.processedMs / props.capture.durationMs) * 100));
  }
  return undefined;
});
const transferred = computed(() =>
  props.capture.downloadedBytes > 0 ? formatByteSize(props.capture.downloadedBytes) : undefined,
);
</script>

<template>
  <article
    class="w-80 overflow-hidden rounded-md border border-stroke-strong bg-surface-panel shadow-control"
    :role="capture.status === 'failed' ? 'alert' : 'status'"
    aria-live="polite"
  >
    <div class="flex min-w-0 items-center gap-3 px-3 py-2.5">
      <div
        class="flex size-10 shrink-0 items-center justify-center rounded-md border border-stroke-default bg-surface-control text-fg-secondary"
      >
        <Clapperboard class="size-5" :stroke-width="1.4" />
      </div>
      <div class="min-w-0 flex-1">
        <p class="truncate text-label font-medium text-fg-primary">{{ capture.title }}</p>
        <p class="mt-0.5 truncate text-caption text-fg-secondary">
          <span>{{
            capture.status === "failed"
              ? t("spaces.importExport.website.captureFailed")
              : t(`spaces.media.import.${capture.status}`)
          }}</span>
          <span v-if="transferred && isByteTransfer" class="tabular-nums">· {{ transferred }}</span>
        </p>
      </div>
      <span v-if="progress !== undefined" class="shrink-0 text-caption text-fg-secondary tabular-nums"
        >{{ progress }}%</span
      >
      <LoaderCircle
        v-else-if="capture.status !== 'failed'"
        class="size-3.5 shrink-0 animate-spin text-fg-accent motion-reduce:animate-none"
        :stroke-width="1.6"
      />
      <AlertCircle v-else class="size-3.5 shrink-0 text-fg-danger" :stroke-width="1.6" />
    </div>
    <div
      v-if="capture.status === 'failed'"
      class="flex items-center justify-end gap-1 border-t border-stroke-default px-2 py-1"
    >
      <UiButton size="sm" variant="secondary" @click="emit('retry')">{{ t("common.retry") }}</UiButton>
      <UiIconButton :label="t('common.close')" size="sm" @click="emit('dismiss')">
        <X class="size-3" />
      </UiIconButton>
    </div>
    <div
      v-else-if="progress !== undefined"
      class="bg-border-default h-0.5"
      role="progressbar"
      :aria-valuenow="progress"
      aria-valuemin="0"
      aria-valuemax="100"
    >
      <div
        class="h-full bg-surface-accent transition-[width] duration-150 ease-standard motion-reduce:transition-none"
        :style="{ width: `${progress}%` }"
      />
    </div>
  </article>
</template>
