<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import {
  LABEL_KEY_BY_UPDATE_STATUS,
  MESSAGE_KEY_BY_UPDATE_FAILURE,
  releaseNotesBody,
  updateTransfer,
} from "../../../foundation/updates/update-presentation";
import { useUpdateStore } from "../../../foundation/updates/update-store";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiMarkdown from "../../../shared/ui/UiMarkdown.vue";
import UiProgress from "../../../shared/ui/UiProgress.vue";
import UiScrollArea from "../../../shared/ui/UiScrollArea.vue";

const emit = defineEmits<{ perform: [action: () => Promise<unknown>] }>();
const { t, locale } = useI18n();
const updates = useUpdateStore();

const state = computed(() => updates.state);
const release = computed(() => (state.value && "release" in state.value ? state.value.release : null));
const failed = computed(() => state.value?.status === "failed");
const status = computed(() => {
  const current = state.value;
  if (!current) return t("updates.connection.loading");
  return t(
    current.status === "failed"
      ? MESSAGE_KEY_BY_UPDATE_FAILURE[current.reason]
      : LABEL_KEY_BY_UPDATE_STATUS[current.status],
  );
});
const transfer = computed(() => (state.value ? updateTransfer(state.value, locale.value) : undefined));
const notes = computed(() => (release.value ? releaseNotesBody(release.value.notes) : ""));
const canCheck = computed(() => state.value && ["idle", "current", "failed"].includes(state.value.status));
</script>

<template>
  <div data-testid="software-update">
    <div class="flex flex-wrap items-center justify-between gap-x-6 gap-y-2 px-4 py-3.5">
      <div class="min-w-40 flex-1">
        <div class="text-body-lg font-medium text-fg-primary">
          {{ release ? t("updates.available.title", { version: release.version }) : t("updates.title") }}
        </div>
        <p
          class="mt-0.5 text-caption leading-normal"
          :class="failed ? 'text-fg-danger' : 'text-fg-secondary'"
          :role="failed ? 'alert' : undefined"
          aria-live="polite"
        >
          {{ status }}
        </p>
        <p
          v-if="state?.status === 'ready' && updates.blockedByTasks"
          class="mt-1 text-caption leading-normal text-fg-warning"
        >
          {{ t("updates.install.blocked") }}
        </p>
      </div>
      <UiButton
        v-if="canCheck"
        data-testid="update-check"
        :disabled="updates.requestPending"
        @click="emit('perform', updates.check)"
        >{{ t("updates.check.action") }}</UiButton
      >
      <UiButton
        v-else-if="state?.status === 'available'"
        data-testid="update-download"
        variant="primary"
        :disabled="updates.requestPending"
        @click="emit('perform', updates.download)"
        >{{ t("updates.download.action") }}</UiButton
      >
      <UiButton
        v-else-if="state?.status === 'downloading'"
        data-testid="update-cancel"
        @click="emit('perform', updates.cancelDownload)"
        >{{ t("updates.download.cancel") }}</UiButton
      >
      <UiButton
        v-else-if="state?.status === 'ready'"
        data-testid="update-install"
        variant="primary"
        :disabled="!updates.canInstall"
        @click="emit('perform', updates.install)"
        >{{ t("updates.install.action") }}</UiButton
      >
      <UiButton v-else-if="!state" @click="emit('perform', updates.initialize)">{{
        t("updates.connection.retry")
      }}</UiButton>
    </div>
    <div v-if="transfer" class="px-4 pb-4" data-testid="update-progress">
      <UiProgress :value="transfer.value" :max="transfer.max" :label="status" />
      <div
        v-if="transfer.amount || transfer.percent"
        class="mt-2 flex items-center justify-between gap-4 text-caption text-fg-tertiary tabular-nums"
      >
        <span>{{ transfer.amount }}</span>
        <span>{{ transfer.percent }}</span>
      </div>
    </div>
    <section v-if="notes" class="border-t border-stroke-divider" data-testid="update-notes">
      <h3 class="px-4 pt-3.5 pb-2 text-caption font-medium text-fg-secondary">
        {{ t("updates.available.notes") }}
      </h3>
      <UiScrollArea viewport-class="max-h-80">
        <UiMarkdown
          :content="notes"
          class="px-4 pb-4 text-fg-secondary [&_h2]:text-fg-primary [&_h3]:text-fg-primary"
        />
      </UiScrollArea>
    </section>
  </div>
</template>
