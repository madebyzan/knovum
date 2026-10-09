<script setup lang="ts">
import { Hand, MousePointer2, SquareDashed, StickyNote, Type } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot, ToolbarSeparator } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import { useNodeOperationGate, type NodeOperationEffect } from "../../canvas/use-node-operation-gate";
import { CANVAS_TOOL_COMMANDS } from "../../commands/canvas-tool-commands";
import SpaceAddMenu from "../creation/SpaceAddMenu.vue";

const tools = useCanvasToolStore();
const nodeOperations = useNodeOperationGate();
const icons = { select: MousePointer2, pan: Hand, note: StickyNote, text: Type, section: SquareDashed };
const { t } = useI18n();
function toolEffect(tool: keyof typeof icons): NodeOperationEffect {
  return tool === "select" || tool === "pan" ? "structure" : "create";
}
</script>

<template>
  <ToolbarRoot
    class="flex h-8 cursor-default items-center gap-0.5 rounded-md border border-stroke-default bg-surface-overlay p-0.5 select-none"
    :aria-label="t('spaces.canvas.tools.title')"
  >
    <template v-for="(toolCommand, index) in CANVAS_TOOL_COMMANDS" :key="toolCommand.tool">
      <ToolbarSeparator v-if="index === 2" class="bg-border-divider mx-1 h-4 w-px" />
      <ToolbarButton as-child>
        <UiIconButton
          :label="t(toolCommand.title)"
          :shortcut="toolCommand.shortcut"
          :active="tools.tool === toolCommand.tool"
          :disabled="!nodeOperations.allows(toolEffect(toolCommand.tool))"
          class="data-[active=true]:bg-surface-selected data-[active=true]:text-fg-accent"
          tooltip-side="bottom"
          size="sm"
          @click="nodeOperations.allows(toolEffect(toolCommand.tool)) && tools.selectTool(toolCommand.tool)"
        >
          <component :is="icons[toolCommand.tool]" class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
      </ToolbarButton>
    </template>
    <ToolbarSeparator class="bg-border-divider mx-1 h-4 w-px" />
    <SpaceAddMenu />
  </ToolbarRoot>
</template>
