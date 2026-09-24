<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiDropdownMenu from "../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import { useCanvasToolStore } from "../canvas/canvas-tool-store";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { useAddMenu } from "../presentation/use-add-menu";
import AgentCreationDialog from "./AgentCreationDialog.vue";
import SpaceKnowledgeTypeManager from "./knowledge/SpaceKnowledgeTypeManager.vue";
import SpaceAddMenuContent from "./SpaceAddMenuContent.vue";
import SpaceCustomKnowledgeObjectDialog from "./SpaceCustomKnowledgeObjectDialog.vue";
import SpacePortalDialog from "./SpacePortalDialog.vue";

const { t } = useI18n();
const tools = useCanvasToolStore();
const nodeOperations = useNodeOperationGate();
const {
  store,
  websiteOpen,
  customObjectOpen,
  typeManagerOpen,
  portalOpen,
  agentCreationOpen,
  website,
  normalizedWebsite,
  addMarkdown,
  addSection,
  addKnowledgeObject,
  addCredential,
  importFromPicker,
  addWebsite,
} = useAddMenu();

async function beginAudioRecording() {
  if (!nodeOperations.allows("create")) return;
  try {
    await store.beginAudioRecording();
  } catch (error) {
    toast.error(t("spaces.media.audioRecordingFailed"), { description: String(error) });
  }
}
</script>

<template>
  <UiDropdownMenu
    side="bottom"
    align="start"
    :side-offset="6"
    @update:open="($event) => $event && nodeOperations.allows('create') && tools.selectTool('select')"
  >
    <template #trigger>
      <ToolbarButton as-child>
        <UiIconButton
          :label="t('spaces.canvas.actions.addContent')"
          :disabled="!nodeOperations.allows('create')"
          size="sm"
          tooltip-side="bottom"
        >
          <Plus class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
        </UiIconButton>
      </ToolbarButton>
    </template>
    <SpaceAddMenuContent
      @markdown="addMarkdown"
      @section="addSection"
      @knowledge="addKnowledgeObject"
      @custom-knowledge="customObjectOpen = true"
      @portal="portalOpen = true"
      @manage-types="typeManagerOpen = true"
      @website="websiteOpen = true"
      @import-file="importFromPicker()"
      @record-audio="beginAudioRecording"
      @credential="addCredential"
      @agent="agentCreationOpen = true"
    />
  </UiDropdownMenu>

  <UiDialog
    v-model:open="websiteOpen"
    :title="t('spaces.transfer.addWebsite')"
    :description="t('spaces.transfer.addWebsiteDescription')"
  >
    <UiInput v-model="website" type="url" placeholder="example.com/article" @keydown.enter="addWebsite" />
    <template #footer>
      <UiButton variant="ghost" @click="websiteOpen = false">{{ t("common.cancel") }}</UiButton>
      <UiButton
        variant="primary"
        :disabled="!normalizedWebsite || !nodeOperations.allows('create')"
        @click="addWebsite"
        >{{ t("common.continue") }}</UiButton
      >
    </template>
  </UiDialog>
  <SpaceCustomKnowledgeObjectDialog v-model:open="customObjectOpen" />
  <SpaceKnowledgeTypeManager v-model:open="typeManagerOpen" />
  <SpacePortalDialog v-model:open="portalOpen" />
  <AgentCreationDialog v-model:open="agentCreationOpen" />
</template>
