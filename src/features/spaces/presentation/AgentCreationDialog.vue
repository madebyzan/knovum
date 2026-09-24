<script setup lang="ts">
import { Bot, Paperclip, Trash2, X } from "@lucide/vue";
import { watch } from "vue";
import { useI18n } from "vue-i18n";

import { isSystemPreviewAvailable } from "../../../foundation/system-preview/system-preview";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
import { useAgentAttachments } from "../agent-creation/use-agent-attachments";
import { useAgentCreation } from "../agent-creation/use-agent-creation";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import SpaceAgentArtifactList from "./SpaceAgentArtifactList.vue";

const open = defineModel<boolean>("open", { required: true });
const { t } = useI18n();
const operations = useNodeOperationGate();
const {
  agents,
  selectedAgent,
  prompt,
  running,
  status,
  pendingPermission,
  artifacts,
  selectedArtifacts,
  agentOptions,
  recoveryOptions,
  sessionId,
  canRun,
  selectedCount,
  run,
  decidePermission,
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
      <section
        v-if="pendingPermission"
        class="space-y-3 rounded-panel border border-warning/40 bg-warning/10 p-3"
        aria-live="assertive"
      >
        <div>
          <p class="text-label text-foreground">{{ pendingPermission.title }}</p>
          <p class="text-caption text-foreground-secondary">
            {{ t("spaces.creation.agentPermissionDescription") }}
          </p>
        </div>
        <div class="flex flex-wrap gap-2">
          <UiButton
            v-for="option in pendingPermission.options"
            :key="option.id"
            size="sm"
            :variant="option.kind.startsWith('allow') ? 'primary' : 'dangerSubtle'"
            @click="decidePermission(option.id)"
          >
            {{ option.label }}
          </UiButton>
          <UiButton size="sm" variant="ghost" @click="decidePermission(null)">
            {{ t("common.cancel") }}
          </UiButton>
        </div>
      </section>
      <SpaceAgentArtifactList
        v-if="artifacts.length"
        :artifacts="artifacts"
        :can-preview="canPreview"
        :selected-artifacts="selectedArtifacts"
        @preview="preview"
      />
    </div>
    <template #footer>
      <UiButton variant="ghost" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton v-if="sessionId && !running" variant="dangerSubtle" @click="discardCurrent">
        <Trash2 class="size-(--knovum-icon-size-sm)" />
        {{ t("spaces.creation.discardAgentArtifacts") }}
      </UiButton>
      <UiButton
        v-if="artifacts.length"
        variant="primary"
        :disabled="!selectedCount || !operations.allows('create')"
        @click="importSelected"
      >
        {{ t("spaces.creation.importAgentArtifacts", { count: selectedCount }) }}
      </UiButton>
      <UiButton v-else variant="primary" :disabled="!canRun || running" @click="run(attachmentPaths)">
        <Bot class="size-(--knovum-icon-size-sm)" />{{ t("spaces.creation.runAgent") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
