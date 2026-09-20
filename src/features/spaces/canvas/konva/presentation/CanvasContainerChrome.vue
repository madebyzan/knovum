<script setup lang="ts">
import { ChevronDown, ChevronRight, Frame, Group, Lock } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceEditorDocument, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasEditorPort, CanvasViewport } from "../../contract/canvas-editor-port";
import type { CanvasScene } from "../scene/canvas-scene";
import CanvasContainerMenu from "./CanvasContainerMenu.vue";
import CanvasContainerTitle from "./CanvasContainerTitle.vue";
import { containerChromeProjection } from "./container-chrome-projection";
const props = defineProps<{
  document: SpaceEditorDocument;
  scene: CanvasScene;
  viewport: CanvasViewport;
  previews: ReadonlyMap<string, CanvasNodeLayoutV1>;
  selected: ReadonlySet<string>;
  visible: ReadonlySet<string>;
  renameId: string | undefined;
  editor: CanvasEditorPort;
  dropTarget: string | null | undefined;
  interactive: boolean;
  reportError(error: unknown): void;
}>();
const emit = defineEmits<{ finishRename: [] }>();
const { t } = useI18n();
const containers = computed(() =>
  containerChromeProjection(
    props.document,
    props.scene,
    props.previews,
    props.viewport,
    props.visible,
    props.selected,
    props.renameId,
  ),
);
function collapse(id: string, collapsed: boolean) {
  void props.editor
    .executeStructureCommand({ kind: "set-section-collapsed", id, collapsed })
    .catch(props.reportError);
}
</script>
<template>
  <div class="pointer-events-none absolute inset-0 z-20 overflow-hidden" data-thumbnail-exclude>
    <div
      v-for="container in containers"
      :key="container.item.id"
      :style="container.style"
      :data-structure-id="container.item.id"
      class="absolute flex h-7 items-center gap-1 rounded-control"
      :class="selected.has(container.item.id) && 'bg-overlay/90'"
    >
      <button
        v-if="container.item.kind === 'section'"
        type="button"
        class="pointer-events-auto grid size-6 shrink-0 place-items-center rounded-control text-foreground-secondary hover:bg-control-hover"
        :aria-label="t(container.collapsed ? 'spaces.structure.expand' : 'spaces.structure.collapse')"
        :aria-expanded="!container.collapsed"
        :disabled="!interactive"
        @pointerdown.stop
        @dblclick.stop
        @click.stop="collapse(container.item.id, !container.collapsed)"
      >
        <component
          :is="container.collapsed ? ChevronRight : ChevronDown"
          class="size-(--knovum-icon-size-xs)"
        />
      </button>
      <component
        :is="container.item.kind === 'section' ? Frame : Group"
        class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
      />
      <CanvasContainerTitle
        :title="container.item.title"
        :editing="renameId === container.item.id && interactive"
        :save="
          (title) =>
            editor.executeStructureCommand({ kind: 'rename-container', id: container.item.id, title })
        "
        @finish="emit('finishRename')"
      />
      <span v-if="viewport.zoom > 0.4" class="shrink-0 text-micro text-foreground-tertiary">{{
        t("spaces.structure.items", { count: container.count })
      }}</span>
      <Lock
        v-if="container.item.locked || container.ancestorLocked"
        class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
      />
      <CanvasContainerMenu
        v-if="interactive && selected.has(container.item.id)"
        :id="container.item.id"
        :kind="container.item.kind"
        :title="container.item.title"
        :count="container.count"
        :locked="container.item.locked"
        :ancestor-locked="container.ancestorLocked"
        :editor="editor"
        :report-error="reportError"
      />
    </div>
    <div
      v-if="dropTarget"
      role="status"
      class="absolute bottom-12 left-1/2 -translate-x-1/2 rounded-control border border-accent bg-overlay px-3 py-1 text-caption text-accent"
    >
      {{
        t("spaces.structure.dropInto", {
          title: containers.find((item) => item.item.id === dropTarget)?.item.title ?? "",
        })
      }}
    </div>
  </div>
</template>
