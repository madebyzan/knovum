<script setup lang="ts">
import { Plus } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { PopoverAnchor, PopoverContent, PopoverPortal, PopoverRoot } from "reka-ui";
import { computed, toRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { POPOVER_SURFACE_CLASSES, POPUP_ANIMATE_CLASSES } from "../../../../shared/ui/ui-classes";
import { DragGripIcon } from "../editor-icons";
import {
  deleteTopLevelBlock,
  duplicateTopLevelBlock,
  insertParagraphBelowAndSlash,
  moveTopLevelBlock,
  selectTopLevelBlockText,
  turnTopLevelBlockInto,
  type TurnIntoBlockKind,
} from "../tiptap-block-actions";
import SpaceTiptapBlockMenuContent from "./SpaceTiptapBlockMenuContent.vue";
import { useSpaceTiptapBlockDrag } from "./use-space-tiptap-block-drag";

const props = defineProps<{
  editor: Editor;
  hoverHost: HTMLElement | null;
  proseHost: HTMLElement | null;
  scrollRoot: HTMLElement | null;
  suppressed?: boolean;
}>();
const emit = defineEmits<{ askAi: [range: { from: number; to: number }]; askInChat: [text: string] }>();
const { t } = useI18n();

const drag = useSpaceTiptapBlockDrag({
  editor: toRef(props, "editor"),
  proseHost: toRef(props, "proseHost"),
  scrollRoot: toRef(props, "scrollRoot"),
  suppressed: toRef(props, "suppressed"),
});
const { hovered, locked, menuOpen, dragging, dropSlot, ghost } = drag;

const activeBlock = computed(() => locked.value ?? hovered.value);

watch(
  () => [props.hoverHost, props.proseHost, props.scrollRoot] as const,
  ([hoverEl, host, scroller], _prev, cleanup) => {
    const pointerTarget = hoverEl ?? host;
    const onKeydown = () => drag.hideHovered();
    const onLeave = () => drag.hideHovered();
    const onScroll = () => drag.hideHovered();
    pointerTarget?.addEventListener("pointermove", drag.onHostPointerMove, { passive: true });
    pointerTarget?.addEventListener("pointerleave", onLeave, { passive: true });
    host?.addEventListener("keydown", onKeydown, { passive: true });
    scroller?.addEventListener("scroll", onScroll, { passive: true });
    cleanup(() => {
      pointerTarget?.removeEventListener("pointermove", drag.onHostPointerMove);
      pointerTarget?.removeEventListener("pointerleave", onLeave);
      host?.removeEventListener("keydown", onKeydown);
      scroller?.removeEventListener("scroll", onScroll);
    });
  },
  { immediate: true },
);

function withActiveBlock(action: (pos: number | undefined) => void) {
  const pos = activeBlock.value?.pos;
  menuOpen.value = false;
  action(pos);
}

function runAskAi() {
  withActiveBlock((pos) => {
    const range = selectTopLevelBlockText(props.editor, pos);
    if (range) emit("askAi", range);
  });
}

function runAskInChat() {
  withActiveBlock((pos) => {
    const range = selectTopLevelBlockText(props.editor, pos);
    if (!range) return;
    const text = props.editor.state.doc.textBetween(range.from, range.to, "\n", "\n").trim();
    if (text) emit("askInChat", text);
  });
}

function runTurnInto(kind: TurnIntoBlockKind) {
  withActiveBlock((pos) => turnTopLevelBlockInto(props.editor, kind, pos));
}
function addBlockBelow() {
  const pos = activeBlock.value?.pos;
  drag.hideHovered();
  insertParagraphBelowAndSlash(props.editor, pos);
}
</script>

<template>
  <PopoverRoot v-model:open="menuOpen">
    <div
      v-if="activeBlock"
      :style="{ top: `${activeBlock.handleTop}px` }"
      class="absolute -left-13 z-10 flex items-center gap-0.5 pr-2 select-none"
      :class="dragging && 'pointer-events-none opacity-0'"
    >
      <button
        type="button"
        draggable="false"
        data-testid="tiptap-add-block"
        :aria-label="t('spaces.richtext.blockMenu.addBelow')"
        :title="t('spaces.richtext.blockMenu.addBelow')"
        class="grid size-5.5 place-items-center rounded-md text-fg-tertiary transition-colors duration-100 ease-standard outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus"
        @mousedown.prevent
        @click.stop="addBlockBelow"
      >
        <Plus class="size-3.5" :stroke-width="1.6" />
      </button>
      <PopoverAnchor as-child>
        <button
          type="button"
          draggable="false"
          data-testid="tiptap-drag-handle"
          :aria-label="t('spaces.richtext.blockMenu.trigger')"
          :title="t('spaces.richtext.blockMenu.trigger')"
          class="grid size-5.5 cursor-grab touch-none place-items-center rounded-md text-fg-tertiary transition-colors duration-100 ease-standard outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus active:cursor-grabbing"
          :class="menuOpen && 'bg-surface-control text-fg-primary'"
          @pointerdown="drag.onHandlePointerDown"
        >
          <DragGripIcon class="size-3.5" />
        </button>
      </PopoverAnchor>
    </div>

    <PopoverPortal>
      <PopoverContent
        side="bottom"
        align="start"
        :side-offset="6"
        :class="[POPUP_ANIMATE_CLASSES, POPOVER_SURFACE_CLASSES, 'z-50 overflow-hidden outline-none']"
        @open-auto-focus.prevent
        @close-auto-focus.prevent
      >
        <SpaceTiptapBlockMenuContent
          @ask-ai="runAskAi"
          @ask-in-chat="runAskInChat"
          @turn-into="runTurnInto"
          @duplicate="withActiveBlock((pos) => duplicateTopLevelBlock(editor, pos))"
          @move-up="withActiveBlock((pos) => moveTopLevelBlock(editor, -1, pos))"
          @move-down="withActiveBlock((pos) => moveTopLevelBlock(editor, 1, pos))"
          @remove="withActiveBlock((pos) => deleteTopLevelBlock(editor, pos))"
        />
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>

  <div
    v-if="dragging && dropSlot"
    data-testid="tiptap-block-drop-line"
    :style="{ top: `${dropSlot.lineTop}px` }"
    class="pointer-events-none absolute inset-x-0 z-30 h-0.5 rounded-full bg-surface-accent"
  >
    <span class="absolute -top-[3px] -left-1 size-2 rounded-full bg-surface-accent" />
  </div>

  <Teleport to="body">
    <div
      v-if="dragging && ghost"
      :style="{ transform: `translate3d(${Math.round(ghost.x)}px, ${Math.round(ghost.y)}px, 0)` }"
      :class="[
        POPOVER_SURFACE_CLASSES,
        'pointer-events-none fixed top-0 left-0 z-50 flex max-w-64 items-center gap-1.5 px-2.5 py-1.5 text-caption text-fg-primary',
      ]"
    >
      <DragGripIcon class="size-3 shrink-0 text-fg-tertiary" />
      <span class="truncate">{{ ghost.label || t("spaces.richtext.paragraph") }}</span>
    </div>
  </Teleport>
</template>
