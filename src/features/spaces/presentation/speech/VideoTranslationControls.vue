<script setup lang="ts">
import { Languages, Square } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, onMounted, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { availableModelTargets, defaultExecutionTarget } from "../../../../foundation/ai/ai-execution";
import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import {
  translationTargetLabelKey,
  translationTargets,
  type TranslationTarget,
} from "../../../../foundation/ai/translation";
import type { ModelTargetSelection } from "../../../../shared/contracts/generated/ModelTargetSelection";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VideoTranslation } from "../../contracts/generated/VideoTranslation";
import type { VideoTranslationProvenance } from "../../contracts/generated/VideoTranslationProvenance";
import type { VideoTranslationSegmentInput } from "../../contracts/generated/VideoTranslationSegmentInput";
import { useVideoTranslationRun } from "../../speech/use-video-translation-run";
import {
  isLocalTranslationTarget,
  persistedTranslationTarget,
  sameTranslationTarget,
  type TranslationModelTarget,
} from "../../speech/video-translation-targets";

const props = defineProps<{
  projectId: string;
  expectedRevision: number;
  segments: readonly VideoTranslationSegmentInput[];
  provenance?: VideoTranslationProvenance | null | undefined;
  disabled: boolean;
}>();
const emit = defineEmits<{ translated: [translation: VideoTranslation]; busy: [value: boolean] }>();
const { t } = useI18n();
const store = useAiSettingsStore();
const { settings, ready } = storeToRefs(store);
const target = shallowRef<TranslationModelTarget>();
const targetLanguage = shallowRef<TranslationTarget>("en");
const localOnly = shallowRef(false);
const { activeRunId, submitting, cancelling, error, restore, start, cancel } = useVideoTranslationRun(
  (translation) => emit("translated", translation),
);
const translating = computed(() => Boolean(activeRunId.value) || submitting.value || cancelling.value);
watch(translating, (value) => emit("busy", value), { immediate: true });
const languageOptions = computed(() =>
  translationTargets.map((value) => ({ value, label: t(translationTargetLabelKey[value]) })),
);
const allTargets = computed(() => availableModelTargets(settings.value, "translation"));
const targets = computed(() =>
  allTargets.value.filter(
    ({ target: candidate }) => !localOnly.value || isLocalTranslationTarget(settings.value, candidate),
  ),
);
const targetValue = computed(() => {
  const selected = target.value;
  return selected
    ? (targets.value.find(({ target: item }) => sameTranslationTarget(item, selected))?.value ?? "")
    : "";
});

watch(
  [ready, targets, localOnly],
  () => {
    const selected = target.value;
    if (
      !ready.value ||
      (selected && targets.value.some(({ target: item }) => sameTranslationTarget(item, selected)))
    )
      return;
    const configured =
      persistedTranslationTarget(settings.value, props.provenance) ??
      defaultExecutionTarget(settings.value, "translation");
    target.value =
      configured?.type === "model" &&
      targets.value.some(({ target: item }) => sameTranslationTarget(item, configured))
        ? configured
        : undefined;
  },
  { immediate: true },
);

watch(
  [() => props.projectId, () => props.expectedRevision],
  ([projectId, revision]) => void restore(projectId, revision),
  { immediate: true },
);

watch(
  [() => props.expectedRevision, () => props.provenance],
  () => {
    const language = props.provenance?.targetLanguage;
    targetLanguage.value = isTranslationTarget(language) ? language : "en";
    localOnly.value = props.provenance?.localOnly ?? false;
  },
  { immediate: true },
);

onMounted(() => store.ensureLoaded());

async function translate() {
  if (!target.value || translating.value || props.segments.length === 0) return;
  const executionTarget: ModelTargetSelection = { ...target.value, temperature: 0 };
  await start({
    jobId: crypto.randomUUID(),
    projectId: props.projectId,
    expectedRevision: props.expectedRevision,
    targetLanguage: targetLanguage.value,
    localOnly: localOnly.value,
    executionTarget,
    segments: [...props.segments],
  });
}

function selectLanguage(value: string) {
  if (isTranslationTarget(value)) targetLanguage.value = value;
}

function isTranslationTarget(value: string | undefined): value is TranslationTarget {
  return translationTargets.includes(value as TranslationTarget);
}
</script>

<template>
  <section class="space-y-3">
    <div class="space-y-3">
      <label class="block space-y-1 text-caption text-fg-secondary">
        <span>{{ t("spaces.speech.translation.language") }}</span>
        <UiSelect
          size="sm"
          :model-value="targetLanguage"
          :label="t('spaces.speech.translation.language')"
          :options="languageOptions"
          :disabled="disabled || translating"
          @update:model-value="selectLanguage"
        />
      </label>
      <label class="block space-y-1 text-caption text-fg-secondary">
        <span>{{ t("spaces.speech.translation.model") }}</span>
        <UiSelect
          size="sm"
          :model-value="targetValue"
          :label="t('spaces.speech.translation.model')"
          :options="targets"
          :placeholder="t('ai.noCompatibleTarget')"
          :disabled="disabled || translating || !ready || targets.length === 0"
          @update:model-value="target = targets.find(({ value }) => value === $event)?.target"
        />
      </label>
    </div>
    <label class="flex items-center gap-2 text-caption text-fg-secondary">
      <UiCheckbox
        v-model="localOnly"
        :label="t('spaces.speech.translation.localOnly')"
        :disabled="disabled || translating"
      />
      {{ t("spaces.speech.translation.localOnly") }}
    </label>
    <p v-if="localOnly && ready && targets.length === 0" class="text-caption text-fg-danger">
      {{ t("spaces.speech.translation.localUnavailable") }}
    </p>
    <div class="flex gap-2">
      <UiButton
        size="sm"
        :disabled="disabled || translating || !target || segments.length === 0"
        @click="translate"
        ><Languages class="size-3.5" />{{ t("spaces.speech.translation.translate") }}</UiButton
      >
      <UiButton v-if="activeRunId" size="sm" variant="dangerSubtle" :disabled="cancelling" @click="cancel">
        <Square class="size-3.5" />{{ t("spaces.speech.translation.cancel") }}
      </UiButton>
    </div>
    <p v-if="error" class="text-caption text-fg-danger" role="alert">
      {{ t("spaces.speech.translation.failed") }}: {{ error }}
    </p>
  </section>
</template>
