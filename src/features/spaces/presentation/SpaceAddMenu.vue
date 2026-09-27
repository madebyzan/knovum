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
import AgentCreationDialog from "./agent-creation/AgentCreationDialog.vue";
import SpaceKnowledgeTypeEditor from "./knowledge/SpaceKnowledgeTypeEditor.vue";
import SpaceAddMenuContent from "./SpaceAddMenuContent.vue";
import SpaceCustomKnowledgeObjectDialog from "./SpaceCustomKnowledgeObjectDialog.vue";
import SpacePortalDialog from "./SpacePortalDialog.vue";
import { useAddMenu } from "./use-add-menu";

const { t } = useI18n();
const tools = useCanvasToolStore();
const nodeOperations = useNodeOperationGate();
const {
  store,
  websiteOpen,
  customObjectOpen,
  typeEditorOpen,
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
    toast.error(t("spaces.recording.startFailed"), { description: String(error) });
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
          :label="t('spaces.canvas.addContent.title')"
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
      @manage-types="typeEditorOpen = true"
      @website="websiteOpen = true"
      @import-file="importFromPicker()"
      @record-audio="beginAudioRecording"
      @credential="addCredential"
      @agent="agentCreationOpen = true"
    />
  </UiDropdownMenu>

  <UiDialog
    v-model:open="websiteOpen"
    :title="t('spaces.importExport.website.add')"
    :description="t('spaces.importExport.website.description')"
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
  <SpaceKnowledgeTypeEditor v-model:open="typeEditorOpen" />
  <SpacePortalDialog v-model:open="portalOpen" />
  <AgentCreationDialog v-model:open="agentCreationOpen" />
</template>
