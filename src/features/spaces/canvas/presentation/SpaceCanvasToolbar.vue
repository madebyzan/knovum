<script setup lang="ts">
import { Hand, MousePointer2, SquareDashed, StickyNote, Type } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot, ToolbarSeparator } from "reka-ui";
import { useI18n } from "vue-i18n";

import { runAppCommandById } from "../../../../foundation/commands/run-app-command";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { CANVAS_TOOL_COMMANDS } from "../../commands/canvas-tool-commands";
import SpaceAddMenu from "../../workspace/presentation/creation/SpaceAddMenu.vue";
import { useCanvasToolStore } from "../canvas-tool-store";
import { useNodeOperationGate, type NodeOperationEffect } from "../use-node-operation-gate";

const props = defineProps<{ navigationEnabled: boolean }>();
const tools = useCanvasToolStore();
const nodeOperations = useNodeOperationGate();
const ICONS = { select: MousePointer2, pan: Hand, note: StickyNote, text: Type, section: SquareDashed };
const { t } = useI18n();
function toolEffect(tool: keyof typeof ICONS): NodeOperationEffect {
  return tool === "select" || tool === "pan" ? "structure" : "create";
}
function canUseTool(tool: keyof typeof ICONS) {
  return nodeOperations.allows(toolEffect(tool)) && (tool !== "pan" || props.navigationEnabled);
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
          :disabled="!canUseTool(toolCommand.tool)"
          class="data-[active=true]:bg-surface-selected data-[active=true]:text-fg-accent"
          tooltip-side="bottom"
          size="sm"
          @click="runAppCommandById(`spaces.tool-${toolCommand.tool}`)"
        >
          <component :is="ICONS[toolCommand.tool]" class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
      </ToolbarButton>
    </template>
    <ToolbarSeparator class="bg-border-divider mx-1 h-4 w-px" />
    <SpaceAddMenu />
  </ToolbarRoot>
</template>
