<script setup lang="ts">
import { Copy, Eye, ExternalLink, Frame, Group, Info, Link2, Sparkles, Ungroup } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import SpaceSelectionMoveMenu from "../SpaceSelectionMoveMenu.vue";
import SpaceCanvasTextFormatBar from "./SpaceCanvasTextFormatBar.vue";
import type { CanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{ actions: CanvasSelectionActions }>();
const { t } = useI18n();
</script>
<template>
  <SpaceCanvasTextFormatBar
    v-if="actions.state.value.textKind && actions.state.value.editable"
    :kind="actions.state.value.textKind"
    v-bind="actions.selectedAppearance.value ? { appearance: actions.selectedAppearance.value } : {}"
    @update="actions.setAppearance"
  />
  <ToolbarButton v-if="actions.state.value.canOpen" as-child>
    <UiIconButton
      :label="t(actions.state.value.opensSystemPreview ? 'spaces.transfer.systemPreview' : 'common.open')"
      size="sm"
      @click="actions.open"
    >
      <Eye v-if="actions.state.value.opensSystemPreview" class="size-(--knovum-icon-size-sm)" /><ExternalLink
        v-else
        class="size-(--knovum-icon-size-sm)"
      />
    </UiIconButton>
  </ToolbarButton>
  <ToolbarButton as-child
    ><UiIconButton :label="t('spaces.navigation.inspect')" size="sm" @click="actions.inspect"
      ><Info class="size-(--knovum-icon-size-sm)" /></UiIconButton
  ></ToolbarButton>
  <ToolbarButton v-if="actions.state.value.canConnect" as-child
    ><UiIconButton :label="t('spaces.canvas.connect')" size="sm" @click="actions.connect"
      ><Link2 class="size-(--knovum-icon-size-sm)" /></UiIconButton
  ></ToolbarButton>
  <ToolbarButton v-if="actions.state.value.content.length" as-child
    ><UiIconButton
      :label="
        t(actions.state.value.section ? 'spaces.assistant.askSection' : 'spaces.assistant.askSelection')
      "
      size="sm"
      @click="actions.ask"
      ><Sparkles class="size-(--knovum-icon-size-sm)" /></UiIconButton
  ></ToolbarButton>
  <ToolbarButton v-if="actions.state.value.roots.length" as-child
    ><UiIconButton
      :label="t('spaces.navigation.duplicateSelection')"
      shortcut="$mod+D"
      size="sm"
      @click="actions.duplicate"
      ><Copy class="size-(--knovum-icon-size-sm)" /></UiIconButton
  ></ToolbarButton>
  <SpaceSelectionMoveMenu v-if="actions.state.value.editable" @move="actions.moveToSpace" />
  <ToolbarButton v-if="actions.state.value.canGroup" as-child
    ><UiIconButton
      :label="t('spaces.structure.groupSelection')"
      shortcut="$mod+G"
      size="sm"
      @click="actions.group"
      ><Group class="size-(--knovum-icon-size-sm)" /></UiIconButton
  ></ToolbarButton>
  <ToolbarButton v-if="actions.state.value.canUngroup" as-child
    ><UiIconButton
      :label="t('spaces.structure.ungroup')"
      shortcut="$mod+Shift+G"
      size="sm"
      @click="actions.ungroup"
      ><Ungroup class="size-(--knovum-icon-size-sm)" /></UiIconButton
  ></ToolbarButton>
  <ToolbarButton v-if="actions.state.value.canSection" as-child
    ><UiIconButton
      :label="t('spaces.structure.sectionFromSelection')"
      size="sm"
      @click="actions.createSection"
      ><Frame class="size-(--knovum-icon-size-sm)" /></UiIconButton
  ></ToolbarButton>
</template>
