<script setup lang="ts">
import { Group, ArrowLeft } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasViewport } from "../../contract/canvas-editor-port";
import type { StructureBounds } from "../../contract/structure-layout";
const props = defineProps<{
  document: SpaceEditorDocument;
  groupId: string | null;
  viewport: CanvasViewport;
  drawing: boolean;
  bounds: StructureBounds | undefined;
  candidates: readonly string[];
  dropTarget: string | null | undefined;
}>();
const emit = defineEmits<{ exit: [] }>();
const { t } = useI18n();
const title = computed(() => props.document.structure.items.find((item) => item.id === props.groupId));
function style(bounds: StructureBounds) {
  return {
    left: `${props.viewport.x + bounds.x * props.viewport.zoom}px`,
    top: `${props.viewport.y + bounds.y * props.viewport.zoom}px`,
    width: `${bounds.width * props.viewport.zoom}px`,
    height: `${bounds.height * props.viewport.zoom}px`,
  };
}
const dropStyle = computed(() => {
  const layout = props.document.layout.items.find((item) => item.itemId === props.dropTarget);
  return layout?.kind === "section"
    ? style({ ...layout.position, ...(layout.collapsed ? { width: 320, height: 64 } : layout.size) })
    : undefined;
});
</script>
<template>
  <div class="pointer-events-none absolute inset-0 z-20 overflow-hidden" data-thumbnail-exclude>
    <div
      v-if="bounds"
      class="absolute rounded-control border-2 border-dashed border-accent bg-accent/5"
      :style="style(bounds)"
    />
    <div
      v-if="dropStyle"
      class="absolute rounded-control border-2 border-accent bg-accent/5"
      :style="dropStyle"
    />
    <div
      v-if="groupId && title?.kind === 'group'"
      class="pointer-events-auto absolute top-3 left-3 flex h-8 items-center gap-2 rounded-control border border-border bg-overlay px-2 shadow-toolbar"
      data-testid="group-edit-scope"
    >
      <button
        type="button"
        :aria-label="t('spaces.structure.exitGroup')"
        class="rounded-control p-1 hover:bg-control-hover"
        @click="emit('exit')"
      >
        <ArrowLeft class="size-(--knovum-icon-size-sm)" />
      </button>
      <Group class="size-(--knovum-icon-size-sm) text-foreground-secondary" /><span class="text-label">{{
        title.title
      }}</span
      ><span class="text-micro text-foreground-tertiary">Esc</span>
    </div>
    <div
      v-if="drawing"
      role="status"
      class="absolute bottom-12 left-1/2 -translate-x-1/2 rounded-control border border-border bg-overlay px-3 py-1 text-caption text-foreground-secondary"
    >
      {{
        bounds
          ? t("spaces.structure.drawCaptured", { count: candidates.length })
          : t("spaces.structure.drawing")
      }}
    </div>
  </div>
</template>
