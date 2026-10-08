<script setup lang="ts">
import { computed, watch, watchEffect } from "vue";
import { useI18n } from "vue-i18n";

import { formatBytes } from "../../../../foundation/format/bytes";
import { useLocalModelsQuery } from "../../../../foundation/runtime/resource-query";
import type { LocalModelOperation } from "../../../../shared/contracts/generated/LocalModelOperation";
import type { LocalModelReferenceTranscript } from "../../../../shared/contracts/generated/LocalModelReferenceTranscript";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VoiceModel } from "../../contracts/generated/VoiceModel";

const model = defineModel<VoiceModel>({ required: true });
const available = defineModel<boolean>("available", { required: true });
const referenceTranscript = defineModel<LocalModelReferenceTranscript>("referenceTranscript", {
  required: true,
});
const props = defineProps<{ sourceKind: "reference" | "designed"; disabled: boolean }>();
const { t } = useI18n();
const catalog = useLocalModelsQuery();
const operation = computed<LocalModelOperation>(() =>
  props.sourceKind === "reference" ? "voiceCloning" : "voiceDesign",
);
const models = computed(() =>
  (catalog.data.value ?? []).filter((entry) => entry.operations.includes(operation.value)),
);
const options = computed(() =>
  models.value.map((entry) => ({
    value: entry.id,
    label: `${entry.name} · ${formatBytes(entry.totalBytes)}`,
    disabled: !entry.downloadAvailable,
  })),
);
let initializedSource: typeof props.sourceKind | undefined;
watch(
  [() => props.sourceKind, () => props.disabled, () => catalog.data.value],
  ([kind, disabled]) => {
    if (disabled || !catalog.data.value) return;
    const choices = models.value;
    if (initializedSource === kind && choices.some((entry) => entry.id === model.value)) return;
    const preferred = choices.find((entry) => entry.selected);
    const smallest = choices
      .filter((entry) => entry.downloadAvailable)
      .reduce<(typeof choices)[number] | undefined>(
        (best, entry) => (!best || entry.totalBytes < best.totalBytes ? entry : best),
        undefined,
      );
    const next = preferred ?? smallest ?? choices[0];
    if (next) model.value = next.id as VoiceModel;
    initializedSource = kind;
  },
  { immediate: true },
);
const selected = computed(() => models.value.find((entry) => entry.id === model.value));
watchEffect(() => {
  available.value = Boolean(selected.value?.downloadAvailable) && !catalog.error.value;
  referenceTranscript.value = selected.value?.referenceTranscript ?? "unused";
});
</script>

<template>
  <div class="space-y-1">
    <UiSelect
      v-model="model"
      size="sm"
      :label="t('spaces.speech.models.label')"
      :options="options"
      :disabled="disabled || catalog.isPending.value"
    />
    <p v-if="catalog.error.value" class="text-caption text-fg-danger" role="alert">
      {{ catalog.error.value.message }}
    </p>
    <UiButton v-if="catalog.error.value" size="sm" variant="ghost" @click="catalog.refetch()">{{
      t("common.retry")
    }}</UiButton>
    <p v-if="selected" class="text-caption text-fg-secondary">
      {{ t(`spaces.speech.models.descriptions.${model}`) }}
    </p>
    <p v-if="selected && !selected.verified" class="text-caption text-fg-tertiary">
      {{ t("spaces.speech.models.firstUse") }}
    </p>
    <p v-if="selected?.unavailableReason" class="text-caption text-fg-secondary">
      {{ t(`settings.ai.localModels.unavailable.${selected.unavailableReason}`) }}
    </p>
  </div>
</template>
