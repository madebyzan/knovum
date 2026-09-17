<script setup lang="ts">
import { Columns3, Rows3, Table2, Trash2 } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { range } from "es-toolkit";
import { ref } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();
const open = ref(false);
const rows = ref(1);
const cols = ref(1);
const size = 8;
const cells = range(0, size * size).map((index) => ({
  row: Math.floor(index / size) + 1,
  col: (index % size) + 1,
}));
function insertTable() {
  props.editor.chain().focus().insertTable({ rows: rows.value, cols: cols.value, withHeaderRow: true }).run();
  open.value = false;
}
function action(run: () => void) {
  run();
  open.value = false;
}
</script>

<template>
  <UiPopover v-model:open="open" align="start" :side-offset="6" class="w-72 p-3">
    <template #trigger>
      <UiIconButton :label="t('editor.table')" size="sm" :active="editor.isActive('table')">
        <Table2 class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
      </UiIconButton>
    </template>

    <template v-if="!editor.isActive('table')">
      <div class="grid grid-cols-8 gap-1" @mouseleave="rows = cols = 1">
        <button
          v-for="cell in cells"
          :key="`${cell.row}-${cell.col}`"
          type="button"
          class="aspect-square rounded-grid-cell border transition-colors"
          :class="
            cell.row <= rows && cell.col <= cols
              ? 'border-accent bg-accent/15'
              : 'border-border bg-field hover:border-border-strong'
          "
          :aria-label="`${cell.row} × ${cell.col}`"
          @mouseenter="
            rows = cell.row;
            cols = cell.col;
          "
          @click="insertTable"
        ></button>
      </div>
      <div class="mt-2 text-center text-caption font-medium text-foreground-secondary">
        {{ rows }} × {{ cols }}
      </div>
    </template>
    <div v-else class="grid gap-1">
      <button
        class="flex h-(--knovum-control-height-md) items-center gap-2 rounded-control px-2.5 text-left text-label text-foreground hover:bg-control-hover"
        type="button"
        @click="action(() => editor.chain().focus().addRowAfter().run())"
      >
        <Rows3 class="size-(--knovum-icon-size-sm)" />{{ t("editor.addTableRow") }}
      </button>
      <button
        class="flex h-(--knovum-control-height-md) items-center gap-2 rounded-control px-2.5 text-left text-label text-foreground hover:bg-control-hover"
        type="button"
        @click="action(() => editor.chain().focus().addColumnAfter().run())"
      >
        <Columns3 class="size-(--knovum-icon-size-sm)" />{{ t("editor.addTableColumn") }}
      </button>
      <button
        class="flex h-(--knovum-control-height-md) items-center gap-2 rounded-control px-2.5 text-left text-label text-danger hover:bg-control-hover"
        type="button"
        @click="action(() => editor.chain().focus().deleteTable().run())"
      >
        <Trash2 class="size-(--knovum-icon-size-sm)" />{{ t("editor.deleteTable") }}
      </button>
    </div>
  </UiPopover>
</template>
