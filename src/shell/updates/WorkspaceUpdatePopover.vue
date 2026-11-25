<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import {
  getUpdateStatusMessageKey,
  releaseNotesBody,
  updateTransfer,
} from "../../foundation/updates/update-presentation";
import type { UpdateMetadata } from "../../shared/contracts/generated/UpdateMetadata";
import type { UpdateState } from "../../shared/contracts/generated/UpdateState";
import UiButton from "../../shared/ui/UiButton.vue";
import UiMarkdown from "../../shared/ui/UiMarkdown.vue";
import UiProgress from "../../shared/ui/UiProgress.vue";
import UiScrollArea from "../../shared/ui/UiScrollArea.vue";
import UiScrollEdgeFade from "../../shared/ui/UiScrollEdgeFade.vue";

const props = defineProps<{
  state: UpdateState;
  release: UpdateMetadata;
  requestPending: boolean;
  blockedByTasks: boolean;
  canInstall: boolean;
}>();

const emit = defineEmits<{
  openSettings: [];
  download: [];
  cancel: [];
  install: [];
}>();

const { t, locale } = useI18n();
const status = computed(() => t(getUpdateStatusMessageKey(props.state)));
const transfer = computed(() => updateTransfer(props.state, locale.value));
const notes = computed(() => releaseNotesBody(props.release.notes));
const heading = computed(() =>
  props.state.status === "ready"
    ? t("updates.ready.title", { version: props.release.version })
    : t("updates.available.title", { version: props.release.version }),
);
</script>

<template>
  <header class="px-4 pt-3.5 pb-2.5">
    <h2 class="m-0 text-label font-semibold tracking-title text-fg-primary">
      {{ heading }}
    </h2>
    <p class="mt-0.5 text-caption leading-normal text-fg-secondary" aria-live="polite">
      {{ status }}
    </p>
    <p
      v-if="state.status === 'ready' && blockedByTasks"
      class="mt-1 text-caption leading-normal text-fg-warning"
    >
      {{ t("updates.install.blocked") }}
    </p>
  </header>

  <div v-if="transfer" class="px-4 pb-3" data-testid="toolbar-update-progress">
    <UiProgress :value="transfer.value" :max="transfer.max" :label="status" />
    <div
      v-if="transfer.amount || transfer.percent"
      class="mt-1.5 flex items-center justify-between gap-4 text-caption-sm text-fg-tertiary tabular-nums"
    >
      <span>{{ transfer.amount }}</span>
      <span>{{ transfer.percent }}</span>
    </div>
  </div>

  <section v-if="notes" class="relative border-t border-stroke-divider" data-testid="toolbar-update-notes">
    <UiScrollArea viewport-class="max-h-56 outline-none">
      <UiMarkdown :content="notes" class="update-notes px-4 py-3 text-fg-secondary" />
    </UiScrollArea>
    <UiScrollEdgeFade surface="raised" edges="bottom" bottom-class="h-6" />
  </section>

  <footer class="flex items-center justify-between gap-3 border-t border-stroke-divider px-4 py-2.5">
    <button
      type="button"
      data-testid="toolbar-update-settings"
      class="rounded-sm py-1 text-caption text-fg-secondary transition-colors duration-100 ease-standard outline-none hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus"
      @click="emit('openSettings')"
    >
      {{ t("updates.indicator.openSettings") }}
    </button>
    <UiButton
      v-if="state.status === 'available'"
      data-testid="toolbar-update-download"
      size="sm"
      variant="primary"
      :disabled="requestPending"
      @click="emit('download')"
    >
      {{ t("updates.download.action") }}
    </UiButton>
    <UiButton
      v-else-if="state.status === 'downloading'"
      data-testid="toolbar-update-cancel"
      size="sm"
      @click="emit('cancel')"
    >
      {{ t("updates.download.cancel") }}
    </UiButton>
    <UiButton
      v-else-if="state.status === 'ready'"
      data-testid="toolbar-update-install"
      size="sm"
      variant="primary"
      :disabled="!canInstall"
      @click="emit('install')"
    >
      {{ t("updates.install.action") }}
    </UiButton>
  </footer>
</template>

<style scoped>
.update-notes :deep(h2),
.update-notes :deep(h3) {
  font-size: var(--text-caption);
  font-weight: 600;
  color: var(--color-fg-primary);
  margin-bottom: 0.35rem;
}
.update-notes :deep(p),
.update-notes :deep(li) {
  font-size: var(--text-caption);
  line-height: 1.6;
}
.update-notes :deep(strong) {
  color: var(--color-fg-primary);
  font-weight: 500;
}
.update-notes :deep(ul),
.update-notes :deep(ol) {
  padding-left: 1.05rem;
}
.update-notes :deep(li::marker) {
  color: var(--color-fg-tertiary);
}
</style>
