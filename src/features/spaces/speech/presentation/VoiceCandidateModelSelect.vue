<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { computed, watch, watchEffect } from "vue";
import { useI18n } from "vue-i18n";

import AiReadinessNotice from "../../../../foundation/ai/AiReadinessNotice.vue";
import { useAiReadiness } from "../../../../foundation/ai/use-ai-readiness";
import { formatBytes } from "../../../../foundation/format/bytes";
import { useLocalModelsQuery } from "../../../../foundation/runtime/use-local-models-query";
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
    label: entry.name,
    description: formatBytes(entry.totalBytes),
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
    // Default to a model this machine is comfortable with; a heavier one stays one click away.
    const smallest = choices
      .filter((entry) => !entry.memoryAdvisoryBytes)
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
const readiness = useAiReadiness(() => [{ kind: "localModel", model: model.value }]);
watchEffect(() => {
  available.value = Boolean(selected.value) && !catalog.error.value;
  referenceTranscript.value = selected.value?.referenceTranscript ?? "unused";
});
</script>

<template>
  <div class="min-w-0 space-y-1">
    <label class="block space-y-1.5 text-caption font-medium text-fg-secondary">
      <span>{{ t("spaces.speech.models.label") }}</span>
      <UiSelect
        v-model="model"
        size="sm"
        :label="t('spaces.speech.models.label')"
        :options="options"
        :disabled="disabled || catalog.isPending.value"
      />
    </label>
    <p v-if="catalog.error.value" class="text-caption text-fg-danger" role="alert">
      {{ catalog.error.value.message }}
    </p>
    <UiButton v-if="catalog.error.value" size="sm" variant="ghost" @click="catalog.refetch()">{{
      t("common.retry")
    }}</UiButton>
    <CollapsibleRoot v-if="selected" class="text-caption-sm text-fg-tertiary">
      <CollapsibleTrigger
        class="inline-flex items-center gap-1 py-1 text-left hover:text-fg-primary focus-visible:outline-stroke-focus"
      >
        {{ t("spaces.speech.models.specs") }}<ChevronDown class="size-3" />
      </CollapsibleTrigger>
      <CollapsibleContent class="space-y-1 pb-1 leading-relaxed">
        <p>{{ formatBytes(selected.totalBytes) }}</p>
        <p>{{ t(`spaces.speech.models.descriptions.${model}`) }}</p>
      </CollapsibleContent>
    </CollapsibleRoot>
    <AiReadinessNotice :readiness="readiness" />
    <p v-if="selected?.memoryAdvisoryBytes" class="text-caption text-fg-secondary">
      {{ t("ai.localModels.memoryAdvisory", { memory: formatBytes(selected.memoryAdvisoryBytes) }) }}
    </p>
  </div>
</template>
