<script setup lang="ts">
import { Bot, Eye, File, Image, Music, Paperclip, Trash2, Video, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import type { AgentArtifact } from "../../../foundation/ai/agents";
import { isSystemPreviewAvailable } from "../../../foundation/system-preview/system-preview";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../shared/ui/UiCheckbox.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
import { useAgentAttachments } from "../agent-creation/use-agent-attachments";
import { useAgentCreation } from "../agent-creation/use-agent-creation";
import { formatByteSize } from "../assets/format-byte-size";

const open = defineModel<boolean>("open", { required: true });
const { t } = useI18n();
const {
  agents,
  selectedAgent,
  prompt,
  running,
  status,
  artifacts,
  selectedArtifacts,
  agentOptions,
  recoveryOptions,
  sessionId,
  canRun,
  selectedCount,
  run,
  discardCurrent,
  importSelected,
  preview,
  selectRecovery,
  openAgentSettings,
} = useAgentCreation(open);
const canPreview = isSystemPreviewAvailable();
const {
  paths: attachmentPaths,
  attachments,
  choose: chooseAttachments,
  remove: removeAttachment,
} = useAgentAttachments(open);

function artifactIcon(kind: AgentArtifact["kind"]) {
  return { image: Image, video: Video, audio: Music, document: File, file: File }[kind];
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    size="wide"
    :title="t('spaces.creation.createWithAgent')"
    :description="t('spaces.creation.createWithAgentDescription')"
  >
    <div v-if="!agents.length" class="rounded-panel border border-border bg-surface p-4">
      <p class="text-body text-foreground-secondary">{{ t("spaces.creation.agentUnavailable") }}</p>
      <UiButton class="mt-3" size="sm" variant="secondary" @click="openAgentSettings">
        {{ t("spaces.creation.configureAgents") }}
      </UiButton>
    </div>
    <div v-else class="space-y-3">
      <UiSelect
        v-if="recoveryOptions.length > 1"
        :model-value="sessionId ?? ''"
        :label="t('spaces.creation.recoveredAgentRuns')"
        :options="recoveryOptions"
        @update:model-value="selectRecovery"
      />
      <UiSelect v-model="selectedAgent" :label="t('spaces.creation.desktopAgent')" :options="agentOptions" />
      <UiTextarea v-model="prompt" :rows="5" :placeholder="t('spaces.creation.agentPrompt')" />
      <div class="flex flex-wrap items-center gap-2">
        <UiButton size="sm" variant="secondary" :disabled="running" @click="chooseAttachments">
          <Paperclip class="size-(--knovum-icon-size-xs)" />{{ t("spaces.creation.addAgentContext") }}
        </UiButton>
        <span class="text-caption text-foreground-secondary">
          {{ t("spaces.creation.agentKnowledgeConnected") }}
        </span>
      </div>
      <div v-if="attachments.length" class="flex flex-wrap gap-1.5">
        <span
          v-for="attachment in attachments"
          :key="attachment.path"
          class="flex max-w-full items-center gap-1 rounded-control bg-control px-2 py-1 text-caption"
        >
          <span class="truncate">{{ attachment.name }}</span>
          <UiButton
            size="sm"
            variant="ghost"
            class="size-4 px-0 shadow-none"
            :aria-label="t('spaces.creation.removeAgentContext', { name: attachment.name })"
            :disabled="running"
            @click="removeAttachment(attachment.path)"
          >
            <X class="size-(--knovum-icon-size-xs)" />
          </UiButton>
        </span>
      </div>
      <p v-if="status" class="line-clamp-3 text-caption text-foreground-secondary">{{ status }}</p>
      <div
        v-if="artifacts.length"
        class="divide-y divide-divider overflow-hidden rounded-panel border border-border"
      >
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
          <component
            :is="artifactIcon(artifact.kind)"
            class="size-(--knovum-icon-size-sm) text-foreground-secondary"
          />
          <div class="min-w-0 flex-1">
            <div class="truncate text-label text-foreground">{{ artifact.name }}</div>
            <div class="truncate text-caption text-foreground-tertiary">
              {{ artifact.relativePath }} · {{ formatByteSize(artifact.size) }}
            </div>
          </div>
          <UiButton v-if="canPreview" size="sm" variant="ghost" @click="preview(artifact)">
            <Eye class="size-(--knovum-icon-size-xs)" />{{ t("spaces.transfer.systemPreview") }}
          </UiButton>
        </div>
      </div>
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton v-if="sessionId && !running" variant="dangerSubtle" @click="discardCurrent">
        <Trash2 class="size-(--knovum-icon-size-sm)" />
        {{ t("spaces.creation.discardAgentArtifacts") }}
      </UiButton>
      <UiButton v-if="artifacts.length" variant="primary" :disabled="!selectedCount" @click="importSelected">
        {{ t("spaces.creation.importAgentArtifacts", { count: selectedCount }) }}
      </UiButton>
      <UiButton v-else variant="primary" :disabled="!canRun || running" @click="run(attachmentPaths)">
        <Bot class="size-(--knovum-icon-size-sm)" />{{ t("spaces.creation.runAgent") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
