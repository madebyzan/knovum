<script setup lang="ts">
import { CornerDownLeft, Trash2 } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { onClickOutside, useTextareaAutosize } from "@vueuse/core";
import { computed, nextTick, onBeforeUnmount, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { isComposing } from "../../../../foundation/dom/composition";
import { POPOVER_SURFACE_CLASSES } from "../../../../shared/ui/ui-classes";
import UiButton from "../../../../shared/ui/UiButton.vue";
import { setMathEditingPos, validateKatexSyntax, type MathEditTarget } from "../tiptap-math";
import {
  applyMathNodeMarkup,
  finalizeMathNodeEdit,
  measureMathPopoverPosition,
  placeCursorAfterMathNode,
  resolveMathNodePos,
  type MathPopoverPosition,
} from "../tiptap-math-editing";

const props = defineProps<{ editor: Editor; target: MathEditTarget | null }>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();

const panelRef = useTemplateRef<HTMLElement>("panelRef");
const textareaRef = useTemplateRef<HTMLTextAreaElement>("textareaRef");
const latex = shallowRef("");
const position = shallowRef<MathPopoverPosition | null>(null);
const initialLatex = shallowRef("");
let committed = false;

useTextareaAutosize({ element: textareaRef, input: latex });

const syntaxError = computed(() =>
  props.target ? validateKatexSyntax(latex.value, props.target.kind === "block") : null,
);

function commitAndClose(refocus = true) {
  if (committed) return;
  committed = true;
  const current = props.target;
  if (!current || props.editor.isDestroyed) {
    emit("close");
    return;
  }
  setMathEditingPos(props.editor.view, null);
  const pos = resolveMathNodePos(props.editor, current);
  const trimmed = latex.value.trim();
  if (pos !== null) {
    finalizeMathNodeEdit(props.editor, pos, initialLatex.value, trimmed);
    placeCursorAfterMathNode(props.editor, pos, Boolean(trimmed), refocus);
  }
  emit("close");
}

function removeEquation() {
  latex.value = "";
  commitAndClose(true);
}

onClickOutside(panelRef, () => {
  if (props.target) commitAndClose(false);
});

watch(
  () => props.target,
  async (next) => {
    if (!next) {
      position.value = null;
      if (!props.editor.isDestroyed) setMathEditingPos(props.editor.view, null);
      return;
    }
    committed = false;
    initialLatex.value = next.latex;
    latex.value = next.latex;
    setMathEditingPos(props.editor.view, next.pos);
    position.value = measureMathPopoverPosition(props.editor, next);
    await nextTick();
    textareaRef.value?.focus();
    textareaRef.value?.select();
  },
  { immediate: true },
);

watch(latex, async (nextValue) => {
  const current = props.target;
  if (!current || committed) return;
  const pos = resolveMathNodePos(props.editor, current);
  if (pos === null) return;
  applyMathNodeMarkup(props.editor, pos, nextValue, false);
  await nextTick();
  position.value = measureMathPopoverPosition(props.editor, current);
});

function handleKeydown(event: KeyboardEvent) {
  if (isComposing(event)) return;
  if (event.key === "Escape" || (event.key === "Enter" && !event.shiftKey)) {
    event.preventDefault();
    commitAndClose(true);
    return;
  }
  if (event.key === "Backspace" && !latex.value && props.target) {
    event.preventDefault();
    removeEquation();
  }
}

onBeforeUnmount(() => {
  if (!props.editor.isDestroyed) setMathEditingPos(props.editor.view, null);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="target && position"
      ref="panelRef"
      data-testid="tiptap-math-popover"
      :style="{ left: `${position.left}px`, top: `${position.top}px` }"
      :class="[POPOVER_SURFACE_CLASSES, 'fixed z-50 w-[440px] max-w-[calc(100vw-24px)] p-1.5']"
    >
      <div class="flex items-end gap-1.5">
        <textarea
          ref="textareaRef"
          v-model="latex"
          data-testid="tiptap-math-input"
          rows="1"
          spellcheck="false"
          :placeholder="
            target.kind === 'block'
              ? t('spaces.richtext.math.blockPlaceholder')
              : t('spaces.richtext.math.inlinePlaceholder')
          "
          class="max-h-40 min-h-7 min-w-0 flex-1 resize-none overflow-y-auto rounded-md bg-transparent px-2 py-1 font-mono text-caption leading-5 text-fg-primary outline-none placeholder:text-fg-tertiary"
          @keydown="handleKeydown"
        />
        <button
          v-if="!target.createdEmpty || latex.trim()"
          type="button"
          data-testid="tiptap-math-delete"
          :aria-label="t('common.delete')"
          :title="t('common.delete')"
          class="grid size-7 shrink-0 place-items-center rounded-md text-fg-tertiary transition-colors duration-100 ease-standard hover:bg-surface-control-hover hover:text-fg-danger"
          @click="removeEquation"
        >
          <Trash2 class="size-3.5" :stroke-width="1.6" />
        </button>
        <UiButton
          data-testid="tiptap-math-save"
          size="sm"
          variant="primary"
          class="shrink-0"
          @click="commitAndClose(true)"
        >
          <span>{{ t("common.done") }}</span>
          <CornerDownLeft class="size-3 opacity-80" :stroke-width="1.75" />
        </UiButton>
      </div>
      <p
        v-if="syntaxError"
        data-testid="tiptap-math-error"
        class="mt-1 truncate border-t border-stroke-divider px-2 pt-1 text-caption-sm text-fg-danger"
      >
        {{ syntaxError }}
      </p>
    </div>
  </Teleport>
</template>
