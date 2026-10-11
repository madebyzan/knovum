<script setup lang="ts">
import { computed, nextTick, toRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { formatBytes } from "../../../../../foundation/format/bytes";
import {
  resolveResourcePreparationPercent,
  formatResourceProgressMessage,
} from "../../../../../foundation/runtime/resource-progress";
import type { LocalModelAvailability } from "../../../../../shared/contracts/generated/LocalModelAvailability";
import UiBadge from "../../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiProgress from "../../../../../shared/ui/UiProgress.vue";
import UiSpinner from "../../../../../shared/ui/UiSpinner.vue";
import { useLocalModelOperations } from "../use-local-model-operations";

const props = defineProps<{ model: LocalModelAvailability }>();
const emit = defineEmits<{ preparationChange: [active: boolean]; failure: [] }>();
const { t } = useI18n();
const { preparation, progress, operation, cancelling, error, busy, select, download, cancel, remove } =
  useLocalModelOperations(toRef(() => props.model));
const statusKey = computed(() =>
  props.model.verified ? "ready" : props.model.installed ? "invalid" : "notInstalled",
);
const modelRef = useTemplateRef<HTMLElement>("model");
const percent = computed(() => progress.value && resolveResourcePreparationPercent(progress.value));
watch(preparation, (handle) => emit("preparationChange", Boolean(handle)), { flush: "sync" });
watch(error, (message) => {
  if (message) emit("failure");
});
watch(
  [preparation, operation, cancelling, () => props.model.selected, () => props.model.installed],
  async () => {
    const modelElement = modelRef.value;
    if (!modelElement) return;
    const focusedElement = document.activeElement;
    if (!focusedElement || !modelElement.contains(focusedElement)) return;
    await nextTick();
    if (document.activeElement !== focusedElement && document.activeElement !== document.body) return;
    if (focusedElement.isConnected && !focusedElement.hasAttribute("disabled")) return;
    const action = modelElement.querySelector<HTMLButtonElement>("button:not(:disabled)");
    (action ?? modelElement).focus();
  },
);
</script>

<template>
  <article
    ref="model"
    tabindex="-1"
    :aria-label="model.name"
    :data-model-id="model.id"
    class="min-h-(--knovum-setting-min-height) space-y-2 px-4 py-3"
  >
    <div class="flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
      <div class="min-w-40 flex-1 space-y-1">
        <div class="flex flex-wrap items-center gap-2">
          <h4 class="text-body-lg font-medium break-words text-fg-primary">{{ model.name }}</h4>
          <UiBadge
            v-if="model.selected"
            size="sm"
            class="bg-surface-control"
            :title="
              t(
                `settings.ai.localModels.${model.capability === 'speechSynthesis' ? 'selectedVoiceModel' : 'selected'}`,
              )
            "
            >{{ t("settings.ai.localModels.defaultModel") }}</UiBadge
          >
        </div>
        <p class="max-w-xl text-caption leading-normal text-fg-secondary">
          {{ t(`settings.ai.localModels.modelDescriptions.${model.id}`) }}
        </p>
        <div class="flex flex-wrap gap-x-3 gap-y-0.5 text-caption text-fg-tertiary">
          <span v-if="!preparation" :class="statusKey === 'invalid' && 'text-fg-warning'">{{
            t(`settings.ai.localModels.${statusKey}`)
          }}</span>
          <span>{{ t("settings.ai.localModels.modelSize", { size: formatBytes(model.totalBytes) }) }}</span>
          <span v-if="model.installedBytes > 0">{{
            t("settings.ai.localModels.installedSize", { size: formatBytes(model.installedBytes) })
          }}</span>
          <span v-if="model.essential">{{ t("settings.ai.localModels.essential") }}</span>
        </div>
        <p v-if="model.fixed" class="text-caption text-fg-tertiary">
          {{ t("settings.ai.localModels.fixedEmbedding") }}
        </p>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <UiButton
          v-if="
            model.selectable && model.verified && model.downloadAvailable && !model.selected && !preparation
          "
          data-testid="model-select"
          size="sm"
          :disabled="busy"
          :aria-busy="operation === 'select'"
          @click="select"
          ><UiSpinner v-if="operation === 'select'" size="sm" />{{
            t("settings.ai.localModels.select")
          }}</UiButton
        >
        <UiButton
          v-if="preparation"
          size="sm"
          variant="ghost"
          :disabled="cancelling"
          :aria-busy="cancelling"
          @click="cancel"
        >
          {{ t(cancelling ? "settings.ai.localModels.cancelling" : "common.cancel") }}
        </UiButton>
        <UiButton
          v-else-if="!model.verified"
          data-testid="model-download"
          size="sm"
          :disabled="busy || !model.downloadAvailable"
          @click="download"
          >{{ t(`settings.ai.localModels.${model.installed ? "redownload" : "download"}`) }}</UiButton
        >
        <UiButton
          v-if="model.installed && !model.essential && !model.selected && !preparation"
          data-testid="model-remove"
          size="sm"
          variant="ghost"
          :disabled="busy"
          :aria-busy="operation === 'remove'"
          @click="remove"
          ><UiSpinner v-if="operation === 'remove'" size="sm" />{{ t("common.remove") }}</UiButton
        >
      </div>
    </div>
    <p v-if="model.unavailableReason" class="text-caption text-fg-secondary">
      {{ t(`ai.localModels.unavailable.${model.unavailableReason}`) }}
    </p>
    <div
      v-if="preparation"
      class="space-y-2 pt-1"
      role="status"
      :aria-live="percent === null || percent === undefined ? 'polite' : 'off'"
    >
      <UiProgress
        v-if="percent !== null && percent !== undefined"
        :value="percent"
        :label="t('settings.ai.localModels.downloading', { name: model.name })"
      />
      <p class="text-caption text-fg-secondary">
        {{ progress ? formatResourceProgressMessage(progress) : t("settings.ai.localModels.preparing") }}
      </p>
    </div>
    <p v-if="error" class="text-caption text-fg-danger" role="alert">{{ error }}</p>
  </article>
</template>
