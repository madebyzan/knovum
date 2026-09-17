<script setup lang="ts">
import { Check, ChevronDown, Maximize2, Minus, Plus, Scan } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiKbd from "../../../../shared/ui/UiKbd.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { SPACE_COMMAND_SHORTCUTS } from "../../commands/space-command-shortcuts";

const { t } = useI18n();
const session = useCanvasEditorSession();
const zoom = computed(() => Math.round((session.viewport.value?.zoom ?? 1) * 100));
const hasSelection = computed(() => Boolean(session.selection.value.nodeIds.length));
const zoomPresets = [25, 50, 75, 100, 125, 200] as const;

function zoomIn() {
  session.activeEditor.value?.zoomIn();
}

function zoomOut() {
  session.activeEditor.value?.zoomOut();
}

function setZoom(percent: number) {
  session.activeEditor.value?.setZoom(percent / 100);
}

function zoomToFit() {
  session.activeEditor.value?.zoomToFit();
}

function zoomToSelection() {
  session.activeEditor.value?.zoomToSelection();
}
</script>

<template>
  <ToolbarRoot
    class="flex h-6 cursor-default items-center gap-0.5 px-0.5 select-none"
    orientation="horizontal"
    :loop="false"
    :aria-label="t('spaces.canvas.zoomOptions')"
  >
    <ToolbarButton as-child>
      <UiIconButton
        :label="t('spaces.canvas.zoomOut')"
        :shortcut="SPACE_COMMAND_SHORTCUTS.zoomOut"
        size="sm"
        class="size-5 rounded-grid-cell"
        @click="zoomOut"
      >
        <Minus class="size-(--knovum-icon-size-xs)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <div class="h-4 w-px bg-border" />
    <UiDropdownMenu side="top" align="start" :side-offset="6">
      <template #trigger>
        <ToolbarButton as-child>
          <button
            type="button"
            class="flex h-5 min-w-12 items-center justify-center gap-0.5 rounded-grid-cell px-1 text-caption font-medium text-foreground-secondary tabular-nums outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring focus-visible:ring-inset active:bg-control-pressed"
            :aria-label="t('spaces.canvas.zoomOptions')"
          >
            {{ zoom }}%
            <ChevronDown class="size-2 text-foreground-tertiary" />
          </button>
        </ToolbarButton>
      </template>
      <UiDropdownMenuItem v-for="preset in zoomPresets" :key="preset" @select="setZoom(preset)">
        <Check class="size-(--knovum-icon-size-sm)" :class="zoom === preset ? 'visible' : 'invisible'" />
        {{ preset }}%
        <UiKbd v-if="preset === 100" :shortcut="SPACE_COMMAND_SHORTCUTS.resetZoom" class="ml-auto" />
      </UiDropdownMenuItem>
      <UiSeparator class="my-1" />
      <UiDropdownMenuItem :disabled="!hasSelection" @select="zoomToSelection">
        <Scan class="size-(--knovum-icon-size-sm)" />{{ t("spaces.canvas.actions.zoomToSelection") }}
        <UiKbd :shortcut="SPACE_COMMAND_SHORTCUTS.zoomToSelection" class="ml-auto" />
      </UiDropdownMenuItem>
      <UiDropdownMenuItem @select="zoomToFit">
        <Maximize2 class="size-(--knovum-icon-size-sm)" />{{ t("spaces.canvas.actions.zoomToFitContent") }}
        <UiKbd :shortcut="SPACE_COMMAND_SHORTCUTS.zoomToFit" class="ml-auto" />
      </UiDropdownMenuItem>
    </UiDropdownMenu>
    <div class="h-4 w-px bg-border" />
    <ToolbarButton as-child>
      <UiIconButton
        :label="t('spaces.canvas.zoomIn')"
        :shortcut="SPACE_COMMAND_SHORTCUTS.zoomIn"
        size="sm"
        class="size-5 rounded-grid-cell"
        @click="zoomIn"
      >
        <Plus class="size-(--knovum-icon-size-xs)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
  </ToolbarRoot>
</template>
