<script setup lang="ts">
import { Ellipsis } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type { CanvasEditorPort } from "../../contract/canvas-editor-port";
const props = defineProps<{
  id: string;
  kind: "group" | "section";
  title: string;
  locked: boolean;
  ancestorLocked: boolean;
  count: number;
  editor: CanvasEditorPort;
  reportError(error: unknown): void;
}>();
const { t } = useI18n();
const run = (work: () => Promise<unknown>) => {
  void work().catch(props.reportError);
};
</script>
<template>
  <span class="pointer-events-auto" @pointerdown.stop @dblclick.stop>
    <UiDropdownMenu align="start" side="bottom">
      <template #trigger
        ><UiIconButton :label="t('spaces.structure.organize')" size="sm"
          ><Ellipsis class="size-(--knovum-icon-size-sm)" /></UiIconButton
      ></template>
      <UiDropdownMenuItem :disabled="locked || ancestorLocked" @select="editor.requestContainerRename(id)">{{
        t("spaces.structure.rename")
      }}</UiDropdownMenuItem>
      <UiDropdownMenuItem
        v-if="kind === 'group'"
        :disabled="locked || ancestorLocked"
        @select="editor.enterGroup(id)"
        >{{ t("spaces.structure.enterGroup") }}</UiDropdownMenuItem
      >
      <UiDropdownMenuItem
        v-else
        :disabled="locked || ancestorLocked || !count"
        @select="run(() => editor.fitSection(id))"
        >{{ t("spaces.structure.fit") }}</UiDropdownMenuItem
      >
      <UiDropdownMenuItem
        :disabled="ancestorLocked"
        @select="
          run(() => editor.executeStructureCommand({ kind: 'set-items-locked', ids: [id], locked: !locked }))
        "
        >{{ t(locked ? "spaces.structure.unlock" : "spaces.structure.lock") }}</UiDropdownMenuItem
      >
      <UiSeparator class="my-1" />
      <UiDropdownMenuItem
        :disabled="locked || ancestorLocked"
        @select="run(() => editor.executeStructureCommand({ kind: 'dissolve-container', id }))"
        >{{
          t(kind === "group" ? "spaces.structure.ungroup" : "spaces.structure.dissolveSection")
        }}</UiDropdownMenuItem
      >
      <UiDropdownMenuItem
        :disabled="locked || ancestorLocked"
        @select="run(() => editor.deleteItems([id]))"
        >{{
          t(count ? "spaces.structure.deleteContainer" : "spaces.structure.deleteEmpty", { title, count })
        }}</UiDropdownMenuItem
      >
    </UiDropdownMenu>
  </span>
</template>
