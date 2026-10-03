<script setup lang="ts">
import { Download, Play, Save, Square, WandSparkles } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, shallowRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
import type {
  SaveSpeechProjectRevisionInput,
  SpeechAudioOutput,
  SpeechGenerationProgress,
  SpeechProjectDetail,
  VoiceProfileSummary,
} from "./contracts";

const props = defineProps<{
  project: DeepReadonly<SpeechProjectDetail>;
  voices: readonly VoiceProfileSummary[];
  preview?: (SpeechAudioOutput & { source: string }) | undefined;
  output?: SpeechAudioOutput | undefined;
  progress?: SpeechGenerationProgress | undefined;
  running: boolean;
}>();
const emit = defineEmits<{
  save: [input: Omit<SaveSpeechProjectRevisionInput, "projectId" | "expectedRevision">];
  preview: [text: string, voiceRevisionId: string];
  generate: [];
  cancel: [];
  export: [path: string];
}>();
const { t } = useI18n();
const text = shallowRef("");
const voiceRevisionId = shallowRef("");
const savedText = shallowRef("");
const savedVoiceRevisionId = shallowRef("");
const voiceOptions = computed(() =>
  props.voices.map((voice) => ({ value: voice.activeRevisionId, label: voice.name })),
);

watch(
  () => props.project,
  (project) => {
    text.value = project.segments.map((segment) => segment.text).join("\n\n");
    savedText.value = text.value;
    voiceRevisionId.value = project.segments[0]?.voiceRevisionId ?? props.voices[0]?.activeRevisionId ?? "";
    savedVoiceRevisionId.value = voiceRevisionId.value;
  },
  { immediate: true },
);
const dirty = computed(
  () => text.value.trim() !== savedText.value.trim() || voiceRevisionId.value !== savedVoiceRevisionId.value,
);
const editable = computed(() => props.project.project.projectType === "short_dubbing");
const canSynthesize = computed(() =>
  Boolean(editable.value && text.value.trim() && voiceRevisionId.value && !props.running),
);

function save() {
  const script = text.value.trim();
  if (!script || !voiceRevisionId.value) return;
  const language =
    props.voices.find((voice) => voice.activeRevisionId === voiceRevisionId.value)?.language ??
    props.project.segments.find((segment) => segment.voiceRevisionId === voiceRevisionId.value)?.language ??
    "zh-CN";
  emit("save", {
    sourceRevision: crypto.randomUUID(),
    pronunciationLexicon: {},
    outputConfig: { format: "wav" },
    segments: [
      {
        id: crypto.randomUUID(),
        text: script,
        voiceRevisionId: voiceRevisionId.value,
        language,
      },
    ],
  });
}

function requestPreview() {
  if (canSynthesize.value)
    emit("preview", [...text.value.trim()].slice(0, 500).join(""), voiceRevisionId.value);
}

function renderSource(path: string) {
  return convertFileSrc(path);
}

function stageLabel(stage: SpeechGenerationProgress["stage"]) {
  if (stage === "preparing") return t("spaces.speech.stages.preparing");
  if (stage === "synthesizing") return t("spaces.speech.stages.synthesizing");
  if (stage === "rendering") return t("spaces.speech.stages.rendering");
  return t("spaces.speech.stages.done");
}
</script>

<template>
  <section class="space-y-2 rounded-md border border-stroke-default p-2.5">
    <div>
      <h2 class="truncate text-label font-medium text-fg-primary">{{ project.project.title }}</h2>
      <p class="text-caption-sm text-fg-tertiary">v{{ project.project.activeRevision }}</p>
    </div>
    <UiTextarea
      v-model="text"
      :rows="6"
      :disabled="!editable"
      :placeholder="t('spaces.speech.scriptPlaceholder')"
    />
    <div class="flex gap-2">
      <UiSelect
        v-model="voiceRevisionId"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.speech.voice')"
        :options="voiceOptions"
        :disabled="!editable || voices.length === 0"
      />
      <UiButton v-if="editable" size="sm" :disabled="!dirty || !canSynthesize" @click="save">
        <Save class="size-3.5" />{{ t("spaces.speech.saveRevision") }}
      </UiButton>
    </div>
    <div class="flex gap-2">
      <UiButton v-if="editable" size="sm" :disabled="!canSynthesize" @click="requestPreview">
        <Play class="size-3.5" />{{ t("spaces.speech.preview") }}
      </UiButton>
      <UiButton
        v-if="!running && project.project.projectType === 'short_dubbing'"
        size="sm"
        variant="primary"
        :disabled="dirty || project.segments.length === 0"
        @click="emit('generate')"
      >
        <WandSparkles class="size-3.5" />{{ t("spaces.speech.generate") }}
      </UiButton>
      <UiButton v-if="running" size="sm" variant="dangerSubtle" @click="emit('cancel')">
        <Square class="size-3.5" />{{ t("spaces.speech.cancel") }}
      </UiButton>
    </div>
    <p v-if="progress && running" class="text-caption-sm text-fg-tertiary">
      {{ stageLabel(progress.stage) }} · {{ progress.completed }}/{{ progress.total }}
    </p>
    <audio v-if="preview" class="h-8 w-full" controls :src="preview.source" />
    <div v-if="output" class="space-y-2 border-t border-stroke-divider pt-2">
      <p class="text-caption font-medium text-fg-secondary">{{ t("spaces.speech.generatedSegments") }}</p>
      <div class="flex items-center gap-2">
        <audio class="h-8 min-w-0 flex-1" controls :src="renderSource(output.path)" />
        <UiButton size="sm" variant="ghost" @click="emit('export', output.path)">
          <Download class="size-3.5" />{{ t("spaces.speech.export") }}
        </UiButton>
      </div>
    </div>
  </section>
</template>
