<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { formatBytes } from "../../../foundation/format/bytes";
import { removeLocalModel, selectLocalModel } from "../../../foundation/runtime/local-resources";
import { startResourcePreparation } from "../../../foundation/runtime/resource-preparation";
import { resourceProgressMessage } from "../../../foundation/runtime/resource-progress";
import { invalidateLocalResources } from "../../../foundation/runtime/use-resource-queries";
import type { LocalModelAvailability } from "../../../shared/contracts/generated/LocalModelAvailability";
import type { ResourcePreparationProgress } from "../../../shared/contracts/generated/ResourcePreparationProgress";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiProgress from "../../../shared/ui/UiProgress.vue";

const props = defineProps<{ model: LocalModelAvailability }>();
const { t } = useI18n();
const preparation = shallowRef<ReturnType<typeof startResourcePreparation>>();
const progress = shallowRef<ResourcePreparationProgress>();
const selecting = shallowRef(false);
const removing = shallowRef(false);
const error = shallowRef<string>();
const statusKey = computed(() =>
  props.model.verified ? "ready" : props.model.installed ? "invalid" : "notInstalled",
);

async function select() {
  selecting.value = true;
  error.value = undefined;
  try {
    await selectLocalModel(props.model.id);
    await invalidateLocalResources();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    selecting.value = false;
  }
}

async function download() {
  error.value = undefined;
  progress.value = undefined;
  const handle = startResourcePreparation(
    { kind: "model", model: props.model.id },
    t("settings.ai.localModels.downloading", { name: props.model.name }),
    (update) => {
      progress.value = update;
    },
  );
  preparation.value = handle;
  try {
    await handle.completion;
  } catch (cause) {
    if (decodeAppError(cause)?.code !== "local_resource_preparation_cancelled")
      error.value = getErrorMessage(cause);
  } finally {
    preparation.value = undefined;
  }
}

async function cancel() {
  try {
    await preparation.value?.cancel();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  }
}

async function remove() {
  if (
    !(await confirmAction({
      title: t("settings.ai.localModels.removeTitle", { name: props.model.name }),
      description: t("settings.ai.localModels.removeDescription"),
      confirmLabel: t("common.remove"),
    }))
  )
    return;
  removing.value = true;
  error.value = undefined;
  try {
    await removeLocalModel(props.model.id);
    await invalidateLocalResources();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    removing.value = false;
  }
}
</script>

<template>
  <article :data-model-id="model.id" class="min-h-(--knovum-setting-min-height) space-y-2 px-4 py-3">
    <div class="flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
      <div class="min-w-40 flex-1 space-y-1">
        <div class="flex flex-wrap items-center gap-2">
          <h3 class="text-body-lg font-medium break-words text-fg-primary">{{ model.name }}</h3>
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
          <span :class="statusKey === 'invalid' && 'text-fg-warning'">{{
            t(`settings.ai.localModels.${statusKey}`)
          }}</span>
          <span>{{ t("settings.ai.localModels.modelSize", { size: formatBytes(model.totalBytes) }) }}</span>
          <span v-if="model.installed">{{
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
          :disabled="selecting || removing"
          @click="select"
          >{{ t("settings.ai.localModels.select") }}</UiButton
        >
        <UiButton v-if="preparation" size="sm" variant="ghost" @click="cancel">{{
          t("common.cancel")
        }}</UiButton>
        <UiButton
          v-else-if="!model.verified"
          data-testid="model-download"
          size="sm"
          :disabled="removing || selecting || !model.downloadAvailable"
          @click="download"
          >{{ t(`settings.ai.localModels.${model.installed ? "redownload" : "download"}`) }}</UiButton
        >
        <UiButton
          v-if="model.installed && !model.essential && !model.selected && !preparation"
          data-testid="model-remove"
          size="sm"
          variant="ghost"
          :disabled="removing || selecting"
          @click="remove"
          >{{ t("common.remove") }}</UiButton
        >
      </div>
    </div>
    <p v-if="model.unavailableReason" class="text-caption text-fg-secondary">
      {{ t(`settings.ai.localModels.unavailable.${model.unavailableReason}`) }}
    </p>
    <div v-if="preparation" class="space-y-2 pt-1" role="status" aria-live="polite">
      <UiProgress
        v-if="progress?.percent !== null && progress?.percent !== undefined"
        :value="progress.percent"
        :label="t('settings.ai.localModels.downloading', { name: model.name })"
      />
      <p class="text-caption text-fg-secondary">
        {{ progress ? resourceProgressMessage(progress) : t("settings.ai.localModels.preparing") }}
      </p>
    </div>
    <p v-if="error" class="text-caption text-fg-danger" role="alert">{{ error }}</p>
  </article>
</template>
