<script setup lang="ts">
import { Languages, Square } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, onMounted, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { availableModelTargets, defaultExecutionTarget } from "../../../../foundation/ai/ai-execution";
import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import {
  LABEL_KEY_BY_TRANSLATION_TARGET,
  TRANSLATION_TARGETS,
  type TranslationTarget,
} from "../../../../foundation/ai/translation";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VideoTranslation } from "../../contracts/generated/VideoTranslation";
import type { VideoTranslationProvenance } from "../../contracts/generated/VideoTranslationProvenance";
import { useVideoTranslationRun } from "../use-video-translation-run";
import {
  persistedTranslationTarget,
  sameTranslationTarget,
  type TranslationModelTarget,
} from "../video-translation-targets";

const props = defineProps<{
  projectId: string;
  expectedRevision: number;
  lineCount: number;
  provenance?: VideoTranslationProvenance | null | undefined;
  /** Commits the script the user is looking at, because a Run translates a saved revision. */
  save: () => Promise<boolean>;
  disabled: boolean;
}>();
const emit = defineEmits<{ translated: [translation: VideoTranslation]; busy: [value: boolean] }>();
const { t } = useI18n();
const store = useAiSettingsStore();
const { settings, ready } = storeToRefs(store);
const target = shallowRef<TranslationModelTarget>();
const targetLanguage = shallowRef<TranslationTarget>("en");
const saving = shallowRef(false);
const { activeRunId, submitting, cancelling, error, restore, start, cancel } = useVideoTranslationRun(
  (translation) => emit("translated", translation),
);
const translating = computed(
  () => Boolean(activeRunId.value) || submitting.value || cancelling.value || saving.value,
);
watch(translating, (value) => emit("busy", value), { immediate: true });
const languageOptions = computed(() =>
  TRANSLATION_TARGETS.map((value) => ({ value, label: t(LABEL_KEY_BY_TRANSLATION_TARGET[value]) })),
);
const targets = computed(() => availableModelTargets(settings.value, "translation"));
const targetValue = computed(() => {
  const selected = target.value;
  return selected
    ? (targets.value.find(({ target: item }) => sameTranslationTarget(item, selected))?.value ?? "")
    : "";
});

watch(
  [ready, targets],
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
  },
  { immediate: true },
);

onMounted(() => store.ensureLoaded());

/**
 * A Run translates the lines of a saved revision, so the script on screen is committed first.
 * That is also why the translated lines come back for review instead of being saved for the user.
 */
async function translate() {
  if (!target.value || translating.value || props.lineCount === 0) return;
  saving.value = true;
  const saved = await props.save().finally(() => {
    saving.value = false;
  });
  if (!saved) return;
  await start({
    jobId: crypto.randomUUID(),
    projectId: props.projectId,
    expectedRevision: props.expectedRevision,
    targetLanguage: targetLanguage.value,
    executionTarget: { ...target.value, temperature: 0 },
  });
}

function selectLanguage(value: string) {
  if (isTranslationTarget(value)) targetLanguage.value = value;
}

function isTranslationTarget(value: string | undefined): value is TranslationTarget {
  return TRANSLATION_TARGETS.includes(value as TranslationTarget);
}
</script>

<template>
  <section class="space-y-2">
    <p class="text-caption text-fg-secondary">{{ t("spaces.speech.translation.description") }}</p>
    <div class="flex flex-wrap items-end gap-2">
      <label class="min-w-32 flex-1 space-y-1 text-caption text-fg-secondary">
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
      <label class="min-w-32 flex-1 space-y-1 text-caption text-fg-secondary">
        <span>{{ t("spaces.speech.translation.model") }}</span>
        <UiSelect
          size="sm"
          :model-value="targetValue"
          :label="t('spaces.speech.translation.model')"
          :options="targets"
          :placeholder="t('ai.target.none')"
          :disabled="disabled || translating || !ready || targets.length === 0"
          @update:model-value="target = targets.find(({ value }) => value === $event)?.target"
        />
      </label>
      <UiButton size="sm" :disabled="disabled || translating || !target || lineCount === 0" @click="translate"
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
