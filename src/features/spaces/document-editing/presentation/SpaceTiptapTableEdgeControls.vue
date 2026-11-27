<script setup lang="ts">
import { Plus } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { useI18n } from "vue-i18n";

import { useSpaceTiptapTableEdge } from "./use-space-tiptap-table-edge";

const props = defineProps<{
  editor: Editor;
  proseHost: HTMLElement | null;
}>();

const { t } = useI18n();
const { box, cancelHide, appendRow, appendColumn } = useSpaceTiptapTableEdge(
  () => props.editor,
  () => props.proseHost,
);
</script>

<template>
  <template v-if="box">
    <button
      type="button"
      data-table-edge-control
      data-testid="table-edge-add-row"
      :aria-label="t('spaces.richtext.table.addRow')"
      :title="t('spaces.richtext.table.addRow')"
      :style="{ left: `${box.left}px`, top: `${box.top + box.height + 3}px`, width: `${box.width}px` }"
      class="absolute z-20 flex h-4 items-center justify-center rounded-sm bg-surface-muted/80 text-fg-tertiary transition-colors duration-100 ease-standard hover:bg-surface-control-hover hover:text-fg-primary"
      @mouseenter="cancelHide"
      @mousedown.prevent
      @click="appendRow"
    >
      <Plus class="size-3" :stroke-width="1.8" />
    </button>
    <button
      type="button"
      data-table-edge-control
      data-testid="table-edge-add-column"
      :aria-label="t('spaces.richtext.table.addColumn')"
      :title="t('spaces.richtext.table.addColumn')"
      :style="{ left: `${box.left + box.width + 3}px`, top: `${box.top}px`, height: `${box.height}px` }"
      class="absolute z-20 flex w-4 items-center justify-center rounded-sm bg-surface-muted/80 text-fg-tertiary transition-colors duration-100 ease-standard hover:bg-surface-control-hover hover:text-fg-primary"
      @mouseenter="cancelHide"
      @mousedown.prevent
      @click="appendColumn"
    >
      <Plus class="size-3" :stroke-width="1.8" />
    </button>
  </template>
</template>
