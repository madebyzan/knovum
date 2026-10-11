<script setup lang="ts">
import { Bot, Paperclip, Trash2, X } from "@lucide/vue";
import { watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { isAssetPreviewAvailable } from "../../assets/asset-preview";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { useAgentAttachments } from "../use-agent-attachments";
import { useAgentCreation } from "../use-agent-creation";
import SpaceAgentArtifactPicker from "./SpaceAgentArtifactPicker.vue";

const open = defineModel<boolean>("open", { required: true });
const { t } = useI18n();
const operations = useNodeOperationGate();
const {
  agents,
  selectedAgent,
  prompt,
  running,
  status,
  artifacts,
  selectedArtifacts,
  setArtifactSelected,
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
const canPreview = isAssetPreviewAvailable();
const {
  paths: attachmentPaths,
  attachments,
  choose: chooseAttachments,
  remove: removeAttachment,
  clear: clearAttachments,
} = useAgentAttachments();

watch(open, (isOpen) => {
  if (!isOpen) clearAttachments();
});
</script>

<template>
  <UiDialog
    v-model:open="open"
    size="wide"
    :title="t('spaces.creation.agent.create')"
    :description="t('spaces.creation.agent.description')"
  >
    <div v-if="!agents.length" class="rounded-panel border border-stroke-default bg-surface-muted p-4">
      <p class="text-body text-fg-secondary">{{ t("spaces.creation.agent.unavailable") }}</p>
      <UiButton class="mt-3" size="sm" variant="secondary" @click="openAgentSettings">
        {{ t("spaces.creation.agent.configure") }}
      </UiButton>
    </div>
    <div v-else class="space-y-3">
      <UiSelect
        v-if="recoveryOptions.length > 1"
        :model-value="sessionId ?? ''"
        :label="t('spaces.creation.agent.recovered.runs')"
        :options="recoveryOptions"
        @update:model-value="selectRecovery"
      />
      <UiSelect
        v-model="selectedAgent"
        :label="t('spaces.creation.agent.desktopAgent')"
        :options="agentOptions"
      />
      <UiTextarea v-model="prompt" :rows="5" :placeholder="t('spaces.creation.agent.prompt')" />
      <div class="flex flex-wrap items-center gap-2">
        <UiButton size="sm" variant="secondary" :disabled="running" @click="chooseAttachments">
          <Paperclip class="size-3" />{{ t("spaces.creation.agent.context.add") }}
        </UiButton>
        <span class="text-caption text-fg-secondary">
          {{ t("spaces.creation.agent.knowledgeConnected") }}
        </span>
      </div>
      <div v-if="attachments.length" class="flex flex-wrap gap-1.5">
        <span
          v-for="attachment in attachments"
          :key="attachment.path"
          class="flex max-w-full items-center gap-1 rounded-md bg-surface-control px-2 py-1 text-caption"
        >
          <span class="truncate">{{ attachment.name }}</span>
          <UiButton
            size="sm"
            variant="ghost"
            class="size-4 px-0 shadow-none"
            :aria-label="t('spaces.creation.agent.context.remove', { name: attachment.name })"
            :disabled="running"
            @click="removeAttachment(attachment.path)"
          >
            <X class="size-3" />
          </UiButton>
        </span>
      </div>
      <p v-if="status" class="line-clamp-3 text-caption text-fg-secondary">{{ status }}</p>
      <SpaceAgentArtifactPicker
        v-if="artifacts.length"
        :artifacts="artifacts"
        :can-preview="canPreview"
        :selected-artifacts="selectedArtifacts"
        @select="setArtifactSelected"
        @preview="preview"
      />
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton v-if="sessionId && !running" variant="dangerSubtle" @click="discardCurrent">
        <Trash2 class="size-3.5" />
        {{ t("spaces.creation.agent.artifacts.discard") }}
      </UiButton>
      <UiButton
        v-if="artifacts.length"
        variant="primary"
        :disabled="!selectedCount || !operations.allows('create')"
        @click="importSelected"
      >
        {{ t("spaces.creation.agent.artifacts.import", { count: selectedCount }) }}
      </UiButton>
      <UiButton v-else variant="primary" :disabled="!canRun || running" @click="run(attachmentPaths)">
        <Bot class="size-3.5" />{{ t("spaces.creation.agent.run") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
