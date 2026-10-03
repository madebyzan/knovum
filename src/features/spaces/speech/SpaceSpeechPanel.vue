<script setup lang="ts">
import { computed } from "vue";

import type { SpeechAudioOutput } from "./contracts";
import SpeechCatalog from "./SpeechCatalog.vue";
import SpeechProjectEditor from "./SpeechProjectEditor.vue";
import SpeechProjectForm from "./SpeechProjectForm.vue";
import { useSpeechWorkspace } from "./use-speech-workspace";
import VoiceProfileForm from "./VoiceProfileForm.vue";

const {
  voices,
  projects,
  selectedProject,
  preview,
  renderOutput,
  generationProgress,
  activeJobId,
  loading,
  error,
  addPreset,
  addProject,
  openProject,
  saveRevision,
  previewSelection,
  generateProject,
  cancelGeneration,
  exportRender,
} = useSpeechWorkspace();

const publishedOutput = computed<SpeechAudioOutput | undefined>(() => {
  const project = selectedProject.value;
  if (!project) return undefined;
  if (renderOutput.value?.revision === project.project.activeRevision) return renderOutput.value.output;
  const revision = project.revision;
  if (
    !revision.outputAssetPath ||
    !revision.outputSha256 ||
    typeof revision.outputSampleRate !== "number" ||
    typeof revision.outputSampleCount !== "number"
  )
    return undefined;
  return {
    path: revision.outputAssetPath,
    sha256: revision.outputSha256,
    sampleRate: revision.outputSampleRate,
    sampleCount: revision.outputSampleCount,
  };
});
</script>

<template>
  <div class="h-full overflow-y-auto p-3">
    <p v-if="error" class="mb-3 rounded-md bg-surface-danger-subtle p-2 text-caption text-fg-danger">
      {{ error }}
    </p>
    <div class="space-y-5" :aria-busy="loading">
      <section class="space-y-2">
        <h2 class="text-label font-medium text-fg-primary">{{ $t("spaces.speech.newVoice") }}</h2>
        <fieldset :disabled="Boolean(activeJobId)">
          <VoiceProfileForm @create="addPreset" />
        </fieldset>
      </section>
      <section class="space-y-2 border-t border-stroke-divider pt-4">
        <h2 class="text-label font-medium text-fg-primary">{{ $t("spaces.speech.newProject") }}</h2>
        <fieldset :disabled="Boolean(activeJobId)">
          <SpeechProjectForm @create="addProject" />
        </fieldset>
      </section>
      <SpeechProjectEditor
        v-if="selectedProject"
        :project="selectedProject"
        :voices="voices"
        :preview="preview"
        :output="publishedOutput"
        :progress="generationProgress"
        :running="Boolean(activeJobId)"
        @save="saveRevision"
        @preview="previewSelection"
        @generate="generateProject"
        @cancel="cancelGeneration"
        @export="exportRender"
      />
      <SpeechCatalog
        :voices="voices"
        :projects="projects"
        :disabled="Boolean(activeJobId)"
        @select-project="openProject"
      />
    </div>
  </div>
</template>
