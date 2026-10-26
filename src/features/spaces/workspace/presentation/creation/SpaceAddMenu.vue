<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../../shared/ui/UiDialog.vue";
import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../../shared/ui/UiInput.vue";
import { useCanvasToolStore } from "../../../canvas/canvas-tool-store";
import { getCanvasEditorSession } from "../../../canvas/editor-session";
import { useNodeOperationGate } from "../../../canvas/use-node-operation-gate";
import SpacePortalDialog from "../../../inspection/presentation/SpacePortalDialog.vue";
import type { GeneratorKind } from "../../../knowledge-derivations/generators/generator-kinds";
import { useCanvasGeneratorStore } from "../../../knowledge-derivations/generators/generator-store";
import SpaceCustomKnowledgeObjectDialog from "../../../knowledge/presentation/SpaceCustomKnowledgeObjectDialog.vue";
import SpaceKnowledgeTypeEditor from "../../../knowledge/presentation/SpaceKnowledgeTypeEditor.vue";
import { useSpaceStore } from "../../../session/space-store";
import SpaceAddMenuContent from "./SpaceAddMenuContent.vue";
import { useAddMenu } from "./use-add-menu";

const { t } = useI18n();
const tools = useCanvasToolStore();
const store = useSpaceStore();
const nodeOperations = useNodeOperationGate();
const generators = useCanvasGeneratorStore();
const editorSession = getCanvasEditorSession();
const {
  websiteOpen,
  customObjectOpen,
  typeEditorOpen,
  portalOpen,
  website,
  normalizedWebsite,
  addMarkdown,
  addSection,
  addKnowledgeObject,
  addCredential,
  importFromPicker,
  addWebsite,
} = useAddMenu();

/** A generator opens where the person is looking. */
function openGenerator(kind: GeneratorKind) {
  const editor = editorSession.activeEditor.value;
  if (editor && nodeOperations.allows("create")) generators.open(kind, { center: editor.visibleCenter() });
}

/** A generator that just opened has taken focus for its prompt; the menu must not take it back. */
function keepGeneratorFocus(event: Event) {
  if (generators.active) event.preventDefault();
}

async function beginAudioRecording() {
  if (!nodeOperations.allows("create")) return;
  try {
    await store.beginAudioRecording();
  } catch (error) {
    toast.error(t("spaces.recording.startFailed"), { description: getErrorMessage(error) });
  }
}
</script>

<template>
  <UiDropdownMenu
    side="bottom"
    align="start"
    :side-offset="6"
    @update:open="($event) => $event && nodeOperations.allows('create') && tools.selectTool('select')"
    @close-auto-focus="keepGeneratorFocus"
  >
    <template #trigger>
      <ToolbarButton as-child>
        <UiIconButton
          :label="t('spaces.canvas.addContent.title')"
          :disabled="!nodeOperations.allows('create')"
          size="sm"
          tooltip-side="bottom"
        >
          <Plus class="size-3.5" :stroke-width="1.6" />
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
      @generate="openGenerator"
    />
  </UiDropdownMenu>

  <UiDialog
    v-model:open="websiteOpen"
    :title="t('spaces.importExport.website.add')"
    :description="t('spaces.importExport.website.description')"
  >
    <UiInput v-model="website" type="url" placeholder="example.com/article" @commit.prevent="addWebsite" />
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
</template>
