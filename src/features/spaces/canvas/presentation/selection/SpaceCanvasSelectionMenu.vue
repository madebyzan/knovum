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
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { SUBMENU_CONTENT_CLASSES } from "../../../../../shared/ui/ui-classes";
import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiKbd from "../../../../../shared/ui/UiKbd.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import { SPACE_COMMAND_SHORTCUTS } from "../../../commands/space-command-shortcuts";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceDerivationMenuItems from "../../../knowledge-derivations/presentation/SpaceDerivationMenuItems.vue";
import SpaceCanvasArrangeMenu from "./SpaceCanvasArrangeMenu.vue";
import SpaceCanvasLayerMenu from "./SpaceCanvasLayerMenu.vue";
import SpaceCanvasParentMenu from "./SpaceCanvasParentMenu.vue";
import SpaceSelectionMoveMenu from "./SpaceSelectionMoveMenu.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  nodeBar?: boolean;
  nodeToolsVisible?: boolean;
  /** The bar already leads with Ask, so the menu does not repeat it. */
  excludeAsk?: boolean;
  /** Actions the caller already offers outside this menu. */
  excludeActions?: readonly CreationIntentId[];
}>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const MENU_ITEM_CLASS =
  "flex h-interaction cursor-default items-center gap-2 rounded-md px-2 text-label text-fg-primary outline-none data-[highlighted]:bg-surface-control-hover data-[disabled]:opacity-40";
const submenu = SUBMENU_CONTENT_CLASSES;
const deleteShortcut = platform() === "macos" ? "Backspace" : "Delete";
const DIRECT_VIDEO_ACTIONS: readonly CreationIntentId[] = ["add-video-subtitles"];
const excludedActions = computed(() => [
  ...(props.excludeActions ?? []),
  ...(props.nodeBar && props.actions.selectedVideoNode.value ? DIRECT_VIDEO_ACTIONS : []),
]);
</script>

<template>
  <UiDropdownMenu side="top" align="center" :side-offset="6" class="min-w-48">
    <template #trigger>
      <ToolbarButton as-child>
        <UiIconButton :label="t('common.more')" :size="nodeBar ? 'md' : 'sm'">
          <MoreHorizontal class="size-3.5" />
        </UiIconButton>
      </ToolbarButton>
    </template>
    <template v-if="$slots['node-tools'] && nodeToolsVisible">
      <slot name="node-tools" />
      <UiSeparator class="my-1" />
    </template>
    <template v-if="actions.state.value.structureNodeIds.length === 1">
      <UiDropdownMenuItem v-if="actions.state.value.canAsk && !excludeAsk" @select="actions.ask">
        <MessageSquareText class="size-3.5" />{{ t("spaces.conversation.askSelection") }}
      </UiDropdownMenuItem>
      <SpaceSelectionMoveMenu
        submenu
        :disabled="!actions.editorProtection.canTransfer.value"
        @move="actions.moveToSpace"
      />
      <UiDropdownMenuItem v-if="actions.state.value.canCreateGroup" @select="actions.createGroup">
        <Group class="size-3.5" />{{ t("spaces.canvas.groups.create") }}
      </UiDropdownMenuItem>
      <UiDropdownMenuItem v-if="actions.state.value.canCreateSection" @select="actions.createSection">
        <Group class="size-3.5" />{{ t("spaces.canvas.sections.create") }}
      </UiDropdownMenuItem>
      <UiSeparator class="my-1" />
    </template>
    <template v-if="actions.state.value.structureNodeIds.length">
      <UiDropdownMenuItem @select="actions.duplicate">
        <Copy class="size-3.5" />{{ t("common.duplicate") }}
        <UiKbd shortcut="$mod+D" class="ml-auto" />
      </UiDropdownMenuItem>
      <UiDropdownMenuItem v-if="actions.state.value.canDissolveGroup" @select="actions.dissolveGroup">
        <Group class="size-3.5" />{{ t("spaces.canvas.groups.ungroup") }}
        <UiKbd shortcut="$mod+Shift+G" class="ml-auto" />
      </UiDropdownMenuItem>
      <template v-if="actions.state.value.section">
        <UiDropdownMenuItem @select="actions.toggleSectionCollapsed">
          {{
            t(
              actions.state.value.section.collapsed
                ? "spaces.canvas.sections.expand"
                : "spaces.canvas.sections.collapse",
            )
          }}
        </UiDropdownMenuItem>
        <UiDropdownMenuItem :disabled="!actions.state.value.canFitSection" @select="actions.fitSection">{{
          t("spaces.canvas.sections.fitToContents")
        }}</UiDropdownMenuItem>
        <UiDropdownMenuItem
          :disabled="!actions.state.value.canDissolveSection"
          @select="actions.dissolveSection"
          >{{ t("spaces.canvas.sections.removeOnly") }}</UiDropdownMenuItem
        >
      </template>
      <UiDropdownMenuItem v-if="actions.state.value.content.length" @select="actions.focus">
        <Focus class="size-3.5" />{{ t("spaces.scenes.focus.selection") }}
      </UiDropdownMenuItem>
      <SpaceDerivationMenuItems
        v-if="actions.selectedNodes.value.length"
        :derivations="derivations"
        :row-class="MENU_ITEM_CLASS"
        :submenu-class="submenu"
        :exclude="excludedActions"
        @invoke="emit('invokeDerivation', $event)"
      />
      <SpaceCanvasArrangeMenu
        :count="actions.state.value.structureNodeIds.length"
        :row-class="MENU_ITEM_CLASS"
        :submenu-class="submenu"
        @align="actions.align"
        @auto-layout="actions.autoLayout"
        @distribute="actions.distribute"
      />
      <SpaceCanvasLayerMenu :row-class="MENU_ITEM_CLASS" :submenu-class="submenu" @move="actions.moveLayer" />
      <SpaceCanvasParentMenu
        :row-class="MENU_ITEM_CLASS"
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
      <UnlockKeyhole v-if="actions.state.value.allLocked" class="size-3.5" />
      <LockKeyhole v-else class="size-3.5" />
      {{ t(actions.state.value.allLocked ? "spaces.canvas.unlock" : "spaces.canvas.lock") }}
      <UiKbd :shortcut="SPACE_COMMAND_SHORTCUTS.lockSelection" class="ml-auto" />
    </UiDropdownMenuItem>
    <UiDropdownMenuItem
      v-if="actions.editorProtection.tabs.value.length"
      @select="actions.editorProtection.goToEditor"
    >
      {{ t("spaces.editor.tabs.goToEditor") }}
    </UiDropdownMenuItem>
    <p v-if="!actions.editorProtection.canRemove.value" class="px-2 py-1 text-caption text-fg-secondary">
      {{ t("spaces.editor.tabs.protected") }}
    </p>
    <UiDropdownMenuItem
      destructive
      :disabled="!actions.editorProtection.canRemove.value"
      @select="actions.remove"
    >
      <Trash2 class="size-3.5" />{{ t("spaces.canvas.selection.delete") }}
      <UiKbd :shortcut="deleteShortcut" class="ml-auto" />
    </UiDropdownMenuItem>
  </UiDropdownMenu>
</template>
