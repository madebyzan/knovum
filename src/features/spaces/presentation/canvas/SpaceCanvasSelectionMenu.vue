<script setup lang="ts">
import { Focus, Frame, LockKeyhole, MoreHorizontal, Trash2, UnlockKeyhole } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceDerivationMenuItems from "../SpaceDerivationMenuItems.vue";
import SpaceCanvasArrangeMenu from "./SpaceCanvasArrangeMenu.vue";
import SpaceCanvasLayerMenu from "./SpaceCanvasLayerMenu.vue";
import type { CanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{ actions: CanvasSelectionActions; derivations: CreationIntentDecisions }>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const menuRow =
  "flex h-(--knovum-compact-item-height) cursor-default items-center gap-2 rounded-control px-2 text-label text-foreground outline-none data-[highlighted]:bg-control-hover data-[disabled]:text-foreground-tertiary";
const submenu = "material-overlay z-50 min-w-48 rounded-panel p-1 ring-1 ring-border";
</script>
<template>
  <UiDropdownMenu side="top" align="center" :side-offset="6" class="min-w-48">
    <template #trigger
      ><ToolbarButton as-child
        ><UiIconButton :label="t('common.more')" size="sm"
          ><MoreHorizontal class="size-(--knovum-icon-size-sm)" /></UiIconButton></ToolbarButton
    ></template>
    <template v-if="actions.state.value.roots.length">
      <UiDropdownMenuItem @select="actions.focus"
        ><Focus class="size-(--knovum-icon-size-sm)" />{{
          t("spaces.navigation.focusSelection")
        }}</UiDropdownMenuItem
      >
      <SpaceDerivationMenuItems
        :derivations="derivations"
        :row-class="menuRow"
        :submenu-class="submenu"
        @invoke="emit('invokeDerivation', $event)"
      />
      <UiDropdownMenuItem @select="actions.toggleLock">
        <UnlockKeyhole
          v-if="actions.state.value.allLocked"
          class="size-(--knovum-icon-size-sm)"
        /><LockKeyhole v-else class="size-(--knovum-icon-size-sm)" />
        {{ t(actions.state.value.allLocked ? "spaces.structure.unlock" : "spaces.structure.lock") }}
      </UiDropdownMenuItem>
      <UiDropdownMenuItem
        v-if="actions.state.value.section"
        :disabled="!actions.state.value.editable || !actions.state.value.content.length"
        @select="actions.fitSection"
        ><Frame class="size-(--knovum-icon-size-sm)" />{{ t("spaces.structure.fit") }}</UiDropdownMenuItem
      >
      <UiDropdownMenuItem
        v-if="actions.state.value.hasParent"
        :disabled="!actions.state.value.editable"
        @select="actions.moveToRoot"
        >{{ t("spaces.structure.moveToRoot") }}</UiDropdownMenuItem
      >
      <SpaceCanvasArrangeMenu
        v-if="actions.state.value.editable && actions.state.value.roots.length > 1"
        :count="actions.state.value.roots.length"
        :row-class="menuRow"
        :submenu-class="submenu"
        @align="actions.align"
        @auto-layout="actions.autoLayout"
        @distribute="actions.distribute"
      />
      <SpaceCanvasLayerMenu
        v-if="actions.state.value.editable"
        :row-class="menuRow"
        :submenu-class="submenu"
        @move="actions.moveLayer"
      />
      <UiDropdownMenuItem
        v-if="actions.state.value.section"
        :disabled="!actions.state.value.editable"
        @select="actions.removeSection"
        >{{ t("spaces.structure.dissolveSection") }}</UiDropdownMenuItem
      >
      <UiSeparator class="my-1" />
    </template>
    <UiDropdownMenuItem
      destructive
      :disabled="actions.state.value.roots.length > 0 && !actions.state.value.editable"
      @select="actions.remove"
      ><Trash2 class="size-(--knovum-icon-size-sm)" />{{
        t("spaces.navigation.deleteSelection")
      }}</UiDropdownMenuItem
    >
  </UiDropdownMenu>
</template>
