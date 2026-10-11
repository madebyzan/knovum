<script setup lang="ts">
import { Focus, Group, LocateFixed, Pencil, Scan } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { runAppCommandById } from "../../../../foundation/commands/run-app-command";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import { getCanvasEditorSession } from "../../canvas/editor-session";
import { revealStructureNode } from "../../canvas/structure-navigation";
import { createStructureIndex } from "../../model/structure/structure-index";
import { useSpaceStore } from "../../session/space-store";
import { canRenameStructureNode } from "../../structure/structure-capabilities";
import { useCanvasSelectionActions } from "../canvas/use-canvas-selection-actions";

const store = useSpaceStore();
const editor = getCanvasEditorSession();
const actions = useCanvasSelectionActions();
const { t } = useI18n();
const roots = computed(() => {
  const index = createStructureIndex(store.editSnapshot.structure);
  return actions.state.value.structureNodeIds.map((id) => index.structureById.get(id)!);
});
const container = computed(() => {
  const root = roots.value[0];
  return roots.value.length === 1 && root?.kind !== "content" ? root : undefined;
});
const canRename = computed(() =>
  canRenameStructureNode(store.editSnapshot, actions.state.value.structureNodeIds),
);
async function reveal() {
  if (!container.value) return;
  try {
    await revealStructureNode(store, editor, container.value.id);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}
</script>

<template>
  <div class="h-full overflow-y-auto overscroll-contain px-4 pt-4 pb-3">
    <template v-if="container">
      <div class="flex items-center gap-1.5 text-caption font-medium text-fg-secondary">
        <Group class="size-3.5" :stroke-width="1.6" />
        {{ t(`spaces.nodeTypes.${container.kind}`) }}
      </div>
      <h2 class="mt-2 text-title leading-snug font-semibold tracking-title wrap-break-word text-fg-primary">
        {{ container.title }}
      </h2>
      <div class="mt-3 flex flex-wrap items-center gap-1.5">
        <UiButton size="sm" variant="ghost" @click="reveal"
          ><LocateFixed class="size-3" />{{ t("spaces.canvas.revealInCanvas") }}</UiButton
        >
        <UiButton v-if="canRename" size="sm" variant="ghost" @click="runAppCommandById('spaces.rename-node')"
          ><Pencil class="size-3" />{{ t("common.rename") }}</UiButton
        >
        <UiButton v-if="actions.state.value.content.length" size="sm" variant="ghost" @click="actions.focus"
          ><Focus class="size-3" />{{ t("spaces.scenes.focus.selection") }}</UiButton
        >
      </div>
      <div v-if="actions.state.value.section" class="mt-3 flex flex-wrap items-center gap-1.5">
        <UiButton size="sm" variant="ghost" @click="actions.toggleSectionCollapsed">{{
          t(
            actions.state.value.section.collapsed
              ? "spaces.canvas.sections.expand"
              : "spaces.canvas.sections.collapse",
          )
        }}</UiButton>
        <UiButton
          size="sm"
          variant="ghost"
          :disabled="!actions.state.value.canFitSection"
          @click="actions.fitSection"
          ><Scan class="size-3" />{{ t("spaces.canvas.sections.fitToContents") }}</UiButton
        >
      </div>
    </template>
    <h2 v-else class="text-title font-semibold text-fg-primary">
      {{ t("spaces.inspector.itemsSelected", { count: roots.length }) }}
    </h2>
    <ul
      v-if="!container"
      class="mt-3 divide-y divide-stroke-divider rounded-section-inner border border-stroke-divider"
    >
      <li v-for="root in roots" :key="root.id" class="truncate px-3 py-2 text-caption text-fg-secondary">
        {{
          root.kind === "content" ? store.nodes.find(({ id }) => id === root.nodeId)?.data.title : root.title
        }}
      </li>
    </ul>
    <p class="mt-4 text-caption text-fg-secondary">
      {{ t("spaces.inspector.containedItems", { count: actions.state.value.content.length }) }}
    </p>
    <ul
      v-if="actions.state.value.content.length"
      class="mt-3 divide-y divide-stroke-divider rounded-section-inner border border-stroke-divider"
    >
      <li
        v-for="node in actions.state.value.content"
        :key="node.id"
        class="truncate px-3 py-2 text-caption text-fg-secondary"
      >
        {{ node.data.title }}
      </li>
    </ul>
  </div>
</template>
