<script setup lang="ts">
import {
  Copy,
  Focus,
  Group,
  LockKeyhole,
  MessageSquareText,
  MoreHorizontal,
  Trash2,
  UnlockKeyhole,
} from "@lucide/vue";
import { platform } from "@tauri-apps/plugin-os";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

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
import SpaceDerivationMenuItems from "../SpaceDerivationMenuItems.vue";
import SpaceSelectionMoveMenu from "../SpaceSelectionMoveMenu.vue";
import SpaceCanvasArrangeMenu from "./SpaceCanvasArrangeMenu.vue";
import SpaceCanvasLayerMenu from "./SpaceCanvasLayerMenu.vue";
import SpaceCanvasParentMenu from "./SpaceCanvasParentMenu.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  promotedDuplicate?: boolean;
}>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const menuRow =
  "flex h-(--knovum-compact-item-height) cursor-default items-center gap-2 rounded-control px-2 text-label text-foreground outline-none data-[highlighted]:bg-control-hover data-[disabled]:text-foreground-tertiary";
const submenu = "material-overlay z-50 min-w-48 rounded-panel p-1 ring-1 ring-border";
const deleteShortcut = platform() === "macos" ? "Backspace" : "Delete";
</script>

<template>
  <UiDropdownMenu side="top" align="center" :side-offset="6" class="min-w-48">
    <template #trigger>
      <ToolbarButton as-child>
        <UiIconButton :label="t('common.more')" size="sm">
          <MoreHorizontal class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </ToolbarButton>
    </template>
    <template v-if="actions.state.value.structureNodeIds.length === 1">
      <UiDropdownMenuItem v-if="actions.state.value.canAsk" @select="actions.ask">
        <MessageSquareText class="size-(--knovum-icon-size-sm)" />{{ t("spaces.assistant.askSelection") }}
      </UiDropdownMenuItem>
      <UiDropdownMenuItem v-if="!promotedDuplicate" @select="actions.duplicate">
        <Copy class="size-(--knovum-icon-size-sm)" />{{ t("spaces.navigation.duplicateSelection") }}
        <UiKbd shortcut="$mod+D" class="ml-auto" />
      </UiDropdownMenuItem>
      <SpaceSelectionMoveMenu submenu @move="actions.moveToSpace" />
      <UiDropdownMenuItem v-if="actions.state.value.canCreateGroup" @select="actions.createGroup">
        <Group class="size-(--knovum-icon-size-sm)" />{{ t("spaces.canvas.groupSelection") }}
      </UiDropdownMenuItem>
      <UiDropdownMenuItem v-if="actions.state.value.canCreateSection" @select="actions.createSection">
        <Group class="size-(--knovum-icon-size-sm)" />{{ t("spaces.canvas.createSection") }}
      </UiDropdownMenuItem>
      <UiSeparator class="my-1" />
    </template>
    <template v-if="actions.state.value.structureNodeIds.length">
      <UiDropdownMenuItem v-if="actions.state.value.canDissolveGroup" @select="actions.dissolveGroup">
        <Group class="size-(--knovum-icon-size-sm)" />{{ t("spaces.canvas.ungroupSelection") }}
        <UiKbd shortcut="$mod+Shift+G" class="ml-auto" />
      </UiDropdownMenuItem>
      <template v-if="actions.state.value.section">
        <UiDropdownMenuItem @select="actions.toggleSectionCollapsed">
          {{
            t(
              actions.state.value.section.collapsed
                ? "spaces.canvas.expandSection"
                : "spaces.canvas.collapseSection",
            )
          }}
        </UiDropdownMenuItem>
        <UiDropdownMenuItem @select="actions.fitSection">{{
          t("spaces.canvas.fitSectionToContents")
        }}</UiDropdownMenuItem>
        <UiDropdownMenuItem @select="actions.dissolveSection">{{
          t("spaces.canvas.removeSectionOnly")
        }}</UiDropdownMenuItem>
      </template>
      <UiDropdownMenuItem v-if="actions.state.value.content.length" @select="actions.focus">
        <Focus class="size-(--knovum-icon-size-sm)" />{{ t("spaces.navigation.focusSelection") }}
      </UiDropdownMenuItem>
      <SpaceDerivationMenuItems
        v-if="actions.selectedNodes.value.length"
        :derivations="derivations"
        :row-class="menuRow"
        :submenu-class="submenu"
        @invoke="emit('invokeDerivation', $event)"
      />
      <SpaceCanvasArrangeMenu
        :count="actions.state.value.structureNodeIds.length"
        :row-class="menuRow"
        :submenu-class="submenu"
        @align="actions.align"
        @auto-layout="actions.autoLayout"
        @distribute="actions.distribute"
      />
      <SpaceCanvasLayerMenu :row-class="menuRow" :submenu-class="submenu" @move="actions.moveLayer" />
      <SpaceCanvasParentMenu
        :row-class="menuRow"
        :submenu-class="submenu"
        :targets="actions.parentTargets.value"
        @reparent="actions.reparent"
      />
      <UiSeparator class="my-1" />
    </template>
    <UiDropdownMenuItem
      v-if="actions.state.value.structureNodeIds.length"
      :disabled="!actions.state.value.canToggleLock"
      @select="actions.toggleLock"
    >
      <UnlockKeyhole v-if="actions.state.value.allLocked" class="size-(--knovum-icon-size-sm)" />
      <LockKeyhole v-else class="size-(--knovum-icon-size-sm)" />
      {{ t(actions.state.value.allLocked ? "spaces.canvas.unlock" : "spaces.canvas.lock") }}
      <UiKbd :shortcut="SPACE_COMMAND_SHORTCUTS.lockSelection" class="ml-auto" />
    </UiDropdownMenuItem>
    <UiDropdownMenuItem destructive @select="actions.remove">
      <Trash2 class="size-(--knovum-icon-size-sm)" />{{ t("spaces.navigation.deleteSelection") }}
      <UiKbd :shortcut="deleteShortcut" class="ml-auto" />
    </UiDropdownMenuItem>
  </UiDropdownMenu>
</template>
