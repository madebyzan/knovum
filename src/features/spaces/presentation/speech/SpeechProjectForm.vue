<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { CreateSpeechProjectRequest } from "../../contracts/generated/CreateSpeechProjectRequest";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { nodeSpeechProjectTypes, type NodeSpeechProjectType } from "../../speech/node-speech";
import { useSpeechMediaSources } from "../../speech/use-speech-media-sources";

const props = defineProps<{ node: SpaceCanvasNode; disabled?: boolean }>();
const emit = defineEmits<{ create: [input: Omit<CreateSpeechProjectRequest, "spaceId" | "sourceNodeId">] }>();
const { t } = useI18n();
const title = shallowRef(props.node.data.title);
const projectType = shallowRef<NodeSpeechProjectType>(nodeSpeechProjectTypes(props.node)[0]!);
const backgroundId = shallowRef("mute");
const { videos, backgrounds } = useSpeechMediaSources();
const sourceVideo = computed(() => videos.value.find((video) => video.id === props.node.id));
const backgroundOptions = computed(() => [
  { value: "mute", label: t("spaces.speech.muteOriginal") },
  ...backgrounds.value.map((background) => ({ value: background.id, label: background.name })),
]);
const canSubmit = computed(
  () => title.value.trim() && (projectType.value !== "video_dubbing" || sourceVideo.value),
);
const typeOptions = computed(() =>
  [
    { value: "short_dubbing", label: t("spaces.speech.types.shortDubbing") },
    { value: "audiobook", label: t("spaces.speech.types.audiobook") },
    { value: "video_dubbing", label: t("spaces.speech.types.videoDubbing") },
  ].filter((option) => nodeSpeechProjectTypes(props.node).includes(option.value as NodeSpeechProjectType)),
);

function submit() {
  if (!canSubmit.value) return;
  const video = sourceVideo.value;
  const background = backgrounds.value.find((source) => source.id === backgroundId.value);
  let outputConfig: CreateSpeechProjectRequest["outputConfig"];
  if (projectType.value === "short_dubbing") outputConfig = { format: "wav", wavPrecision: "pcm16" };
  else if (projectType.value === "video_dubbing") {
    if (!video) return;
    outputConfig = {
      format: "mp4",
      video: video.source,
      background: background?.source ?? null,
      backgroundSeparation: null,
      voiceGainDb: 0,
      backgroundGainDb: -12,
      translation: null,
      preserveOriginalOutsideDialogue: false,
      sourceDialogueIntervals: [],
      retainOriginalTrack: true,
    };
  } else {
    outputConfig = {
      format: "m4b",
      sampleRate: 48_000,
      loudnessLufs: -20,
      truePeakDb: -2,
      paragraphPauseMs: 250,
      chapterPauseMs: 1000,
      wavPrecision: "pcm16",
      metadata: { title: title.value.trim(), author: "", narrator: "", cover: null },
      chapters: [],
      roles: [],
    };
  }
  emit("create", {
    title: title.value.trim(),
    projectType: projectType.value,
    outputConfig,
  });
  title.value = "";
}
</script>

<template>
  <form class="space-y-2" @submit.prevent="submit">
    <UiInput v-model="title" size="sm" :placeholder="t('spaces.speech.projectName')" :disabled="disabled" />
    <UiSelect
      v-if="typeOptions.length > 1"
      v-model="projectType"
      size="sm"
      :label="t('spaces.speech.projectType')"
      :options="typeOptions"
      :disabled="disabled"
    />
    <template v-if="projectType === 'video_dubbing'">
      <p class="text-caption text-fg-secondary">{{ node.data.title }}</p>
      <UiSelect
        v-model="backgroundId"
        size="sm"
        :label="t('spaces.speech.backgroundAudio')"
        :options="backgroundOptions"
        :disabled="disabled"
      />
      <p class="text-caption text-fg-tertiary">{{ t("spaces.speech.videoSourcesHint") }}</p>
    </template>
    <UiButton class="w-full" size="sm" variant="primary" type="submit" :disabled="disabled || !canSubmit">
      {{ t("spaces.speech.createProject") }}
    </UiButton>
  </form>
</template>
