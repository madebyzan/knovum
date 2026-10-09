<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { CreateSpeechProjectRequest } from "../../contracts/generated/CreateSpeechProjectRequest";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { NodeSpeechProjectType } from "../../speech/node-speech";
import { useSpeechMediaSources } from "../../speech/use-speech-media-sources";

const props = defineProps<{
  node: SpaceCanvasNode;
  projectType: NodeSpeechProjectType;
  disabled?: boolean;
}>();
const emit = defineEmits<{ create: [input: Omit<CreateSpeechProjectRequest, "spaceId" | "sourceNodeId">] }>();
const { t } = useI18n();
const title = shallowRef(props.node.data.title);
const projectType = computed(() => props.projectType);
const backgroundId = shallowRef("mute");
const { videos, backgrounds } = useSpeechMediaSources();
const sourceVideo = computed(() => videos.value.find((video) => video.id === props.node.id));
const backgroundOptions = computed(() => [
  { value: "mute", label: t("spaces.speech.videoDubbing.muteOriginal") },
  ...backgrounds.value.map((background) => ({ value: background.id, label: background.name })),
]);
const canSubmit = computed(
  () => title.value.trim() && (projectType.value !== "video_dubbing" || sourceVideo.value),
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
}
</script>

<template>
  <form class="space-y-3" @submit.prevent="submit">
    <label class="block space-y-1.5 text-caption font-medium text-fg-secondary">
      <span>{{ t("spaces.speech.project.name") }}</span>
      <UiInput
        v-model="title"
        size="sm"
        :placeholder="t('spaces.speech.project.name')"
        :aria-label="t('spaces.speech.project.name')"
        :disabled="disabled"
      />
    </label>
    <template v-if="projectType === 'video_dubbing'">
      <label class="block space-y-1.5 text-caption font-medium text-fg-secondary">
        <span>{{ t("spaces.speech.videoDubbing.backgroundAudio") }}</span>
        <UiSelect
          v-model="backgroundId"
          size="sm"
          :label="t('spaces.speech.videoDubbing.backgroundAudio')"
          :options="backgroundOptions"
          :disabled="disabled"
        />
      </label>
      <p class="text-caption leading-relaxed text-fg-tertiary">
        {{ t("spaces.speech.videoDubbing.sourcesHint") }}
      </p>
    </template>
    <div class="flex justify-end border-t border-stroke-divider pt-3">
      <UiButton
        size="sm"
        variant="primary"
        type="submit"
        class="bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90"
        :disabled="disabled || !canSubmit"
      >
        {{ t("common.create") }}
      </UiButton>
    </div>
  </form>
</template>
