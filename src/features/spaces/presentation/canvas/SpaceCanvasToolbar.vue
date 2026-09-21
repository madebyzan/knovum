<script setup lang="ts">
import { Hand, MousePointer2, StickyNote, Type } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot, ToolbarSeparator } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import { CANVAS_TOOL_COMMANDS } from "../../commands/canvas-tool-commands";
import SpaceAddMenu from "../workspace/SpaceAddMenu.vue";

const tools = useCanvasToolStore();
const icons = { select: MousePointer2, pan: Hand, note: StickyNote, text: Type };
const { t } = useI18n();
</script>

<template>
  <ToolbarRoot
    class="flex h-8 cursor-default items-center gap-0.5 rounded-control border border-border bg-overlay p-0.5 shadow-toolbar select-none"
    :aria-label="t('spaces.canvas.tools.title')"
  >
    <template v-for="(item, index) in CANVAS_TOOL_COMMANDS" :key="item.tool">
      <ToolbarSeparator v-if="index === 2" class="mx-1 h-4 w-px bg-divider" />
      <ToolbarButton as-child>
        <UiIconButton
          :label="t(item.title)"
          :shortcut="item.shortcut"
          :active="tools.tool === item.tool"
          class="data-[active=true]:bg-selected data-[active=true]:text-accent"
          tooltip-side="bottom"
          size="sm"
          @click="tools.selectTool(item.tool)"
        >
          <component :is="icons[item.tool]" class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
        </UiIconButton>
      </ToolbarButton>
    </template>
    <ToolbarSeparator class="mx-1 h-4 w-px bg-divider" />
    <SpaceAddMenu />
  </ToolbarRoot>
</template>
