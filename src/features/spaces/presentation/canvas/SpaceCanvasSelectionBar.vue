<script setup lang="ts">
import {
  Copy,
  Eye,
  ExternalLink,
  Focus,
  Frame,
  Group,
  Info,
  Link2,
  LockKeyhole,
  MoreHorizontal,
  Sparkles,
  Trash2,
  Ungroup,
  UnlockKeyhole,
} from "@lucide/vue";
import { platform } from "@tauri-apps/plugin-os";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiKbd from "../../../../shared/ui/UiKbd.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import { SPACE_COMMAND_SHORTCUTS } from "../../commands/space-command-shortcuts";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceDerivationMenuItems from "../derivations/SpaceDerivationMenuItems.vue";
import SpaceSelectionMoveMenu from "../workspace/SpaceSelectionMoveMenu.vue";
import SpaceCanvasArrangeMenu from "./SpaceCanvasArrangeMenu.vue";
import SpaceCanvasLayerMenu from "./SpaceCanvasLayerMenu.vue";
import SpaceCanvasTextFormatBar from "./SpaceCanvasTextFormatBar.vue";
import { useCanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{ derivations: CreationIntentDecisions }>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const actions = useCanvasSelectionActions();
const { t } = useI18n();
const menuRow =
  "flex h-(--knovum-compact-item-height) cursor-default items-center gap-2 rounded-control px-2 text-label text-foreground outline-none data-[highlighted]:bg-control-hover data-[disabled]:text-foreground-tertiary";
const submenu = "material-overlay z-50 min-w-48 rounded-panel p-1 ring-1 ring-border";
const deleteShortcut = platform() === "macos" ? "Backspace" : "Delete";
const selectedTextKind = computed<"note" | "text" | undefined>(() => {
  if (actions.selectedNodes.value.length !== 1) return undefined;
  const kind = actions.selectedNodes.value[0]?.data.kind;
  return kind === "note" || kind === "text" ? kind : undefined;
});
</script>

<template>
  <ToolbarRoot
    v-if="actions.visible.value"
    class="flex h-7 cursor-default items-center gap-0.5 rounded-control border border-border bg-overlay select-none"
    orientation="horizontal"
    :loop="false"
    :aria-label="t('spaces.navigation.selection')"
  >
    <UiBadge v-if="actions.selectedNodes.value.length > 1" size="sm" class="mx-1">
      {{ actions.selectedNodes.value.length }}
    </UiBadge>
    <SpaceCanvasTextFormatBar
      v-if="selectedTextKind"
      :kind="selectedTextKind"
      v-bind="actions.selectedAppearance.value ? { appearance: actions.selectedAppearance.value } : {}"
      @update="actions.setAppearance"
    />
    <ToolbarButton v-if="actions.state.value.canOpen" as-child>
      <UiIconButton
        :label="t(actions.state.value.opensSystemPreview ? 'spaces.transfer.systemPreview' : 'common.open')"
        size="sm"
        @click="actions.open"
      >
        <Eye
          v-if="actions.state.value.opensSystemPreview"
          class="size-(--knovum-icon-size-sm)"
          :stroke-width="1.55"
        />
        <ExternalLink v-else class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton as-child>
      <UiIconButton :label="t('spaces.navigation.inspect')" size="sm" @click="actions.inspect">
        <Info class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.state.value.canConnect" as-child>
      <UiIconButton :label="t('spaces.canvas.connect')" size="sm" @click="actions.connect">
        <Link2 class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.selectedNodes.value.length" as-child>
      <UiIconButton
        :label="
          t(actions.state.value.section ? 'spaces.assistant.askSection' : 'spaces.assistant.askSelection')
        "
        size="sm"
        @click="actions.ask"
      >
        <Sparkles class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.state.value.content.length && !actions.state.value.section" as-child>
      <UiIconButton
        :label="t('spaces.navigation.duplicateSelection')"
        shortcut="$mod+D"
        size="sm"
        @click="actions.duplicate"
      >
        <Copy class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <SpaceSelectionMoveMenu
      v-if="actions.state.value.content.length && !actions.state.value.section"
      @move="actions.moveToSpace"
    />
    <ToolbarButton v-if="actions.state.value.hasSectionMembership && !actions.state.value.section" as-child>
      <UiIconButton :label="t('spaces.canvas.ungroupSelection')" size="sm" @click="actions.ungroup">
        <Ungroup class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-else-if="actions.state.value.content.length && !actions.state.value.section" as-child>
      <UiIconButton :label="t('spaces.canvas.createSection')" size="sm" @click="actions.createSection">
        <Group class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
    <UiDropdownMenu side="top" align="center" :side-offset="6" class="min-w-48">
      <template #trigger>
        <ToolbarButton as-child>
          <UiIconButton :label="t('common.more')" size="sm">
            <MoreHorizontal class="size-(--knovum-icon-size-sm)" />
          </UiIconButton>
        </ToolbarButton>
      </template>
      <template v-if="actions.selectedNodes.value.length">
        <UiDropdownMenuItem @select="actions.focus">
          <Focus class="size-(--knovum-icon-size-sm)" />{{
            t(actions.state.value.section ? "spaces.canvas.focusSection" : "spaces.navigation.focusSelection")
          }}
        </UiDropdownMenuItem>
        <SpaceDerivationMenuItems
          :derivations="derivations"
          :row-class="menuRow"
          :submenu-class="submenu"
          @invoke="emit('invokeDerivation', $event)"
        />
        <UiDropdownMenuItem @select="actions.toggleLock">
          <UnlockKeyhole v-if="actions.state.value.allLocked" class="size-(--knovum-icon-size-sm)" />
          <LockKeyhole v-else class="size-(--knovum-icon-size-sm)" />
          {{ t(actions.state.value.allLocked ? "spaces.canvas.unlock" : "spaces.canvas.lock") }}
          <UiKbd :shortcut="SPACE_COMMAND_SHORTCUTS.lockSelection" class="ml-auto" />
        </UiDropdownMenuItem>
        <UiDropdownMenuItem v-if="actions.state.value.section" @select="actions.fitSection">
          <Frame class="size-(--knovum-icon-size-sm)" />{{ t("spaces.canvas.fitSectionToContents") }}
        </UiDropdownMenuItem>
        <SpaceCanvasArrangeMenu
          v-if="!actions.state.value.section"
          :count="actions.state.value.content.length"
          :row-class="menuRow"
          :submenu-class="submenu"
          @align="actions.align"
          @auto-layout="actions.autoLayout"
          @distribute="actions.distribute"
        />
        <SpaceCanvasLayerMenu :row-class="menuRow" :submenu-class="submenu" @move="actions.moveLayer" />
        <UiDropdownMenuItem v-if="actions.state.value.section" @select="actions.removeSection">
          <Trash2 class="size-(--knovum-icon-size-sm)" />{{ t("spaces.canvas.removeSectionOnly") }}
        </UiDropdownMenuItem>
        <UiSeparator class="my-1" />
      </template>
      <UiDropdownMenuItem destructive @select="actions.remove">
        <Trash2 class="size-(--knovum-icon-size-sm)" />{{ t("spaces.navigation.deleteSelection") }}
        <UiKbd :shortcut="deleteShortcut" class="ml-auto" />
      </UiDropdownMenuItem>
    </UiDropdownMenu>
  </ToolbarRoot>
</template>
