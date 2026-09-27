<script setup lang="ts">
import { Eye, File, Image, Music, Video } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import type { AgentArtifact } from "../../agent-creation/agent-artifacts";
import { formatByteSize } from "../../assets/format-byte-size";

defineProps<{
  artifacts: readonly AgentArtifact[];
  canPreview: boolean;
  selectedArtifacts: Set<string>;
}>();
const emit = defineEmits<{ preview: [artifact: AgentArtifact] }>();
const { t } = useI18n();
const icons = { image: Image, video: Video, audio: Music, document: File, file: File };
</script>

<template>
  <div class="divide-y divide-stroke-divider overflow-hidden rounded-panel border border-stroke-default">
    <div
      v-for="artifact in artifacts"
      :key="artifact.relativePath"
      class="flex items-center gap-3 px-3 py-2.5"
    >
      <UiCheckbox
        :model-value="selectedArtifacts.has(artifact.relativePath)"
        :label="artifact.name"
        @update:model-value="
          $event
            ? selectedArtifacts.add(artifact.relativePath)
            : selectedArtifacts.delete(artifact.relativePath)
        "
      />
      <component :is="icons[artifact.kind]" class="size-3.5 text-fg-secondary" />
      <div class="min-w-0 flex-1">
        <div class="truncate text-label text-fg-primary">{{ artifact.name }}</div>
        <div class="truncate text-caption text-fg-tertiary">
          {{ artifact.relativePath }} · {{ formatByteSize(artifact.size) }}
        </div>
      </div>
      <UiButton v-if="canPreview" size="sm" variant="ghost" @click="emit('preview', artifact)">
        <Eye class="size-3" />{{ t("spaces.importExport.preview.system") }}
      </UiButton>
    </div>
  </div>
</template>
