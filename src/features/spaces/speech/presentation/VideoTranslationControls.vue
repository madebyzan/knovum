<script setup lang="ts">
import { Languages, Square } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, onMounted, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { availableModelTargets } from "../../../../foundation/ai/ai-execution";
import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import AiReadinessNotice from "../../../../foundation/ai/AiReadinessNotice.vue";
import {
  LABEL_KEY_BY_TRANSLATION_TARGET,
  TRANSLATION_TARGETS,
  type TranslationTarget,
} from "../../../../foundation/ai/translation";
import { useAiReadiness } from "../../../../foundation/ai/use-ai-readiness";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VideoTranslation } from "../../contracts/generated/VideoTranslation";
import type { VideoTranslationProvenance } from "../../contracts/generated/VideoTranslationProvenance";
import { useVideoTranslationRun } from "../use-video-translation-run";
import {
  parseTranslationTerms,
  resolveInitialTarget,
  sameTranslationTarget,
  type TranslationModelTarget,
} from "../video-translation-targets";

const props = defineProps<{
  projectId: string;
  expectedRevision: number;
  lineCount: number;
  provenance?: VideoTranslationProvenance | null | undefined;
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
const termsKey = computed(() => `knovum:speech:terms:${props.projectId}`);
const preservedTerms = shallowRef(localStorage.getItem(termsKey.value) ?? "");
watch(preservedTerms, (val) => localStorage.setItem(termsKey.value, val));
watch(
  () => props.projectId,
  () => (preservedTerms.value = localStorage.getItem(termsKey.value) ?? ""),
);
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
const readiness = useAiReadiness([{ kind: "modelChoice", purpose: "translation" }]);
const targetValue = computed(() =>
  target.value
    ? (targets.value.find(({ target: item }) => sameTranslationTarget(item, target.value!))?.value ?? "")
    : "",
);

watch(
  [ready, targets],
  () => {
    if (
      !ready.value ||
      (target.value && targets.value.some(({ target: item }) => sameTranslationTarget(item, target.value!)))
    )
      return;
    target.value = resolveInitialTarget(settings.value, targets.value, props.provenance);
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
    const lang = props.provenance?.targetLanguage;
    if (TRANSLATION_TARGETS.includes(lang as TranslationTarget))
      targetLanguage.value = lang as TranslationTarget;
  },
  { immediate: true },
);

onMounted(() => store.ensureLoaded());

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
    terms: parseTranslationTerms(preservedTerms.value),
  });
}

function selectLanguage(value: string) {
  if (TRANSLATION_TARGETS.includes(value as TranslationTarget))
    targetLanguage.value = value as TranslationTarget;
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
    <label class="block space-y-1 text-caption text-fg-secondary">
      <span>{{ t("spaces.speech.translation.terms") }}</span>
      <UiInput
        size="sm"
        v-model="preservedTerms"
        :placeholder="t('spaces.speech.translation.termsPlaceholder')"
        :disabled="disabled || translating"
      />
    </label>
    <AiReadinessNotice :readiness="readiness" />
    <p v-if="error" class="text-caption text-fg-danger" role="alert">
      {{ t("spaces.speech.translation.failed") }}: {{ error }}
    </p>
  </section>
</template>
