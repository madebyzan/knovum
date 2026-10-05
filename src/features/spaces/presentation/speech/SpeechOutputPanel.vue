<script setup lang="ts">
import { Download } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useTemplateRef, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SpeechAudioOutput } from "../../contracts/generated/SpeechAudioOutput";
import type { SpeechProjectSnapshot } from "../../contracts/generated/SpeechProjectSnapshot";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";
import SpeechSignalInspection from "./SpeechSignalInspection.vue";

const props = defineProps<{ project: DeepReadonly<SpeechProjectSnapshot>; output: SpeechAudioOutput }>();
const emit = defineEmits<{ export: [path: string] }>();
const player = useTemplateRef("player");
const { t } = useI18n();
</script>

<template>
  <section class="space-y-2 border-t border-stroke-divider pt-2">
    <p class="text-caption font-medium text-fg-secondary">
      {{
        t(
          project.project.projectType === "video_dubbing"
            ? "spaces.speech.generatedVideo"
            : "spaces.speech.generatedSegments",
        )
      }}
    </p>
    <div class="flex items-center gap-2">
      <SpeechMediaPlayer
        ref="player"
        :kind="project.project.projectType === 'video_dubbing' ? 'video' : 'audio'"
        :source="convertFileSrc(output.path)"
      />
      <UiButton size="sm" variant="ghost" @click="emit('export', props.output.path)"
        ><Download class="size-3.5" />{{ t("spaces.speech.export") }}</UiButton
      >
    </div>
    <SpeechSignalInspection
      :project-id="project.project.id"
      :revision-id="project.revision.id"
      @seek="player?.seekTo($event)"
    />
  </section>
</template>
