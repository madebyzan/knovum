<script setup lang="ts">
import { Combine, Split, Trash2 } from "@lucide/vue";
import type { Editor as CoreEditor } from "@tiptap/core";
import type { Editor } from "@tiptap/vue-3";
import { BubbleMenu } from "@tiptap/vue-3/menus";
import { shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { POPOVER_SURFACE_CLASSES } from "../../../../shared/ui/ui-classes";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import {
  TableColDeleteIcon,
  TableColLeftIcon,
  TableColRightIcon,
  TableHeaderRowIcon,
  TableRowAboveIcon,
  TableRowBelowIcon,
  TableRowDeleteIcon,
} from "../table-icons";
import type { SpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";

const props = defineProps<{ editor: Editor; writing: SpaceTiptapWritingTools }>();
const { t } = useI18n();

const canMergeCells = shallowRef(false);
const canSplitCell = shallowRef(false);

watch(
  () => props.editor,
  (editor, _prev, cleanup) => {
    const sync = () => {
      const inTable = !editor.isDestroyed && editor.isActive("table");
      canMergeCells.value = inTable && Boolean(editor.can().mergeCells?.());
      canSplitCell.value = inTable && Boolean(editor.can().splitCell?.());
    };
    sync();
    editor.on("selectionUpdate", sync);
    editor.on("update", sync);
    cleanup(() => {
      editor.off("selectionUpdate", sync);
      editor.off("update", sync);
    });
  },
  { immediate: true },
);

function shouldShowTableMenu({ editor }: { editor: CoreEditor }) {
  return (
    editor.isEditable &&
    props.writing.phase.value === "closed" &&
    editor.isActive("table") &&
    !editor.isActive("link") &&
    !editor.view.dom.classList.contains("resize-cursor") &&
    editor.state.selection.empty
  );
}

function getTableVirtualElement() {
  const dom = props.editor.view.domAtPos(props.editor.state.selection.from).node;
  const element = dom instanceof Element ? dom : dom.parentElement;
  const anchor = element?.closest(".tableWrapper") ?? element?.closest("table");
  if (!anchor) return null;
  return { getBoundingClientRect: () => anchor.getBoundingClientRect() };
}
</script>

<template>
  <BubbleMenu
    :editor="editor"
    plugin-key="tableBubbleMenu"
    :should-show="shouldShowTableMenu"
    :get-referenced-virtual-element="getTableVirtualElement"
    :options="{ placement: 'top-start', offset: 8 }"
  >
    <div
      data-testid="tiptap-table-menu"
      :class="[POPOVER_SURFACE_CLASSES, 'z-40 flex items-center gap-0.5 p-1']"
    >
      <UiIconButton
        :label="t('spaces.richtext.table.addRowBefore')"
        tooltip
        size="sm"
        @click="editor.chain().focus().addRowBefore().run()"
      >
        <TableRowAboveIcon class="size-3.5" :stroke-width="1.65" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.richtext.table.addRowAfter')"
        tooltip
        size="sm"
        @click="editor.chain().focus().addRowAfter().run()"
      >
        <TableRowBelowIcon class="size-3.5" :stroke-width="1.65" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.richtext.table.deleteRow')"
        tooltip
        size="sm"
        @click="editor.chain().focus().deleteRow().run()"
      >
        <TableRowDeleteIcon class="size-3.5" :stroke-width="1.65" />
      </UiIconButton>
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <UiIconButton
        :label="t('spaces.richtext.table.addColumnBefore')"
        tooltip
        size="sm"
        @click="editor.chain().focus().addColumnBefore().run()"
      >
        <TableColLeftIcon class="size-3.5" :stroke-width="1.65" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.richtext.table.addColumnAfter')"
        tooltip
        size="sm"
        @click="editor.chain().focus().addColumnAfter().run()"
      >
        <TableColRightIcon class="size-3.5" :stroke-width="1.65" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.richtext.table.deleteColumn')"
        tooltip
        size="sm"
        @click="editor.chain().focus().deleteColumn().run()"
      >
        <TableColDeleteIcon class="size-3.5" :stroke-width="1.65" />
      </UiIconButton>
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <UiIconButton
        :label="t('spaces.richtext.table.toggleHeaderRow')"
        tooltip
        size="sm"
        @click="editor.chain().focus().toggleHeaderRow().run()"
      >
        <TableHeaderRowIcon class="size-3.5" :stroke-width="1.65" />
      </UiIconButton>
      <UiIconButton
        v-if="canMergeCells"
        :label="t('spaces.richtext.table.mergeCells')"
        tooltip
        size="sm"
        @click="editor.chain().focus().mergeCells().run()"
      >
        <Combine class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <UiIconButton
        v-if="canSplitCell"
        :label="t('spaces.richtext.table.splitCell')"
        tooltip
        size="sm"
        @click="editor.chain().focus().splitCell().run()"
      >
        <Split class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <UiIconButton
        :label="t('spaces.richtext.table.delete')"
        tooltip
        size="sm"
        class="text-fg-danger"
        @click="editor.chain().focus().deleteTable().run()"
      >
        <Trash2 class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
    </div>
  </BubbleMenu>
</template>
