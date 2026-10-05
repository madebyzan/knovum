<script setup lang="ts">
import { AudioLines, BookOpenText, Mic2, Video } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import type { SpeechProjectOverview } from "../../contracts/generated/SpeechProjectOverview";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";

defineProps<{
  voices: readonly VoiceLibraryRevision[];
  projects: readonly SpeechProjectOverview[];
}>();
const emit = defineEmits<{ selectProject: [projectId: string] }>();
const { t } = useI18n();
function projectIcon(project: SpeechProjectOverview) {
  if (project.projectType === "short_dubbing") return AudioLines;
  if (project.projectType === "video_dubbing") return Video;
  return project.projectType === "audiobook" ? BookOpenText : AudioLines;
}
function projectTypeLabel(project: SpeechProjectOverview) {
  if (project.projectType === "short_dubbing") return t("spaces.speech.types.shortDubbing");
  if (project.projectType === "video_dubbing") return t("spaces.speech.types.videoDubbing");
  if (project.projectType === "audiobook") return t("spaces.speech.types.audiobook");
  return t("spaces.speech.unsupportedProject");
}
</script>

<template>
  <div class="space-y-5">
    <section>
      <h3 class="mb-2 text-caption font-medium text-fg-secondary">{{ t("spaces.speech.voices") }}</h3>
      <p v-if="voices.length === 0" class="text-caption text-fg-tertiary">
        {{ t("spaces.speech.noVoices") }}
      </p>
      <div v-else class="space-y-1">
        <div
          v-for="voice in voices"
          :key="voice.revisionId"
          class="flex items-center gap-2 rounded-md bg-surface-control px-2 py-1.5"
        >
          <Mic2 class="size-3.5 text-fg-tertiary" />
          <div class="min-w-0 flex-1">
            <p class="truncate text-label text-fg-primary">{{ voice.name }}</p>
            <p class="truncate text-caption-sm text-fg-tertiary">
              {{ voice.language }} · {{ voice.engineId }}
            </p>
          </div>
        </div>
      </div>
    </section>
    <section>
      <h3 class="mb-2 text-caption font-medium text-fg-secondary">{{ t("spaces.speech.projects") }}</h3>
      <p v-if="projects.length === 0" class="text-caption text-fg-tertiary">
        {{ t("spaces.speech.noProjects") }}
      </p>
      <div v-else class="space-y-1">
        <button
          v-for="project in projects"
          :key="project.id"
          type="button"
          class="flex w-full items-center gap-2 rounded-md bg-surface-control px-2 py-1.5 text-left hover:bg-surface-control-hover"
          @click="emit('selectProject', project.id)"
        >
          <component :is="projectIcon(project)" class="size-3.5 text-fg-tertiary" />
          <div class="min-w-0 flex-1">
            <p class="truncate text-label text-fg-primary">{{ project.title }}</p>
            <p class="text-caption-sm text-fg-tertiary">
              {{ projectTypeLabel(project) }}
              · v{{ project.activeRevision }} · {{ project.segmentCount }}
            </p>
          </div>
        </button>
      </div>
    </section>
  </div>
</template>
