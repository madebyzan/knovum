<script setup lang="ts">
import { Sigma, Trash2 } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import DOMPurify from "dompurify";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import { renderKatexHtml } from "../../../../shared/ui/math-rendering";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();

const open = ref(false);
const mode = ref<"inline" | "block">("inline");
const latex = ref("");
const editingPos = ref<number | null>(null);

const previewHtml = computed(() => {
  const source = latex.value.trim();
  return source ? DOMPurify.sanitize(renderKatexHtml(source, mode.value === "block")) : "";
});

function openForInsert() {
  editingPos.value = null;
  mode.value = "inline";
  latex.value = "";
  open.value = true;
}

function resolveMathNodePos(
  target: HTMLElement,
): { pos: number; kind: "inline" | "block"; latex: string } | null {
  const { view, state } = props.editor;
  const rawPos = view.posAtDOM(target, 0);
  for (const candidate of [rawPos, rawPos - 1]) {
    if (candidate < 0) continue;
    const node = state.doc.nodeAt(candidate);
    if (node?.type.name === "inlineMath" || node?.type.name === "blockMath") {
      return {
        pos: candidate,
        kind: node.type.name === "blockMath" ? "block" : "inline",
        latex: String(node.attrs.latex ?? ""),
      };
    }
  }
  return null;
}

function handleEditorClick(event: MouseEvent) {
  if (!props.editor.isEditable) return;
  const target =
    event.target instanceof HTMLElement
      ? event.target.closest<HTMLElement>('[data-type="inline-math"], [data-type="block-math"]')
      : null;
  if (!target) return;
  const hit = resolveMathNodePos(target);
  if (!hit) return;
  editingPos.value = hit.pos;
  mode.value = hit.kind;
  latex.value = hit.latex;
  open.value = true;
}

function submit() {
  const value = latex.value.trim();
  if (!value) return;
  const chain = props.editor.chain().focus();
  if (editingPos.value !== null) {
    if (mode.value === "block") chain.updateBlockMath({ pos: editingPos.value, latex: value }).run();
    else chain.updateInlineMath({ pos: editingPos.value, latex: value }).run();
  } else if (mode.value === "block") {
    chain.insertBlockMath({ latex: value }).run();
  } else {
    chain.insertInlineMath({ latex: value }).run();
  }
  open.value = false;
  editingPos.value = null;
}

function remove() {
  if (editingPos.value === null) return;
  const chain = props.editor.chain().focus();
  if (mode.value === "block") chain.deleteBlockMath({ pos: editingPos.value }).run();
  else chain.deleteInlineMath({ pos: editingPos.value }).run();
  open.value = false;
  editingPos.value = null;
}

onMounted(() => {
  props.editor.view.dom.addEventListener("click", handleEditorClick);
});
onBeforeUnmount(() => {
  props.editor.view.dom.removeEventListener("click", handleEditorClick);
});
</script>

<template>
  <UiPopover v-model:open="open" align="start" :side-offset="6" class="w-80 p-3">
    <template #trigger>
      <UiIconButton
        :label="t('spaces.richtext.math.title')"
        tooltip
        size="sm"
        :active="editor.isActive('inlineMath') || editor.isActive('blockMath')"
        @click="openForInsert"
      >
        <Sigma class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
    </template>

    <div class="grid gap-2.5">
      <div class="flex items-center justify-between gap-2">
        <span class="text-label font-medium text-fg-primary">
          {{ editingPos === null ? t("spaces.richtext.math.title") : t("spaces.richtext.math.edit") }}
        </span>
        <div v-if="editingPos === null" class="flex items-center gap-1 rounded-md bg-surface-muted p-0.5">
          <button
            type="button"
            class="rounded-sm px-2 py-0.5 text-caption-sm font-medium transition-colors"
            :class="mode === 'inline' ? 'bg-surface-elevated text-fg-primary shadow-xs' : 'text-fg-secondary'"
            @click="mode = 'inline'"
          >
            {{ t("spaces.richtext.math.inline") }}
          </button>
          <button
            type="button"
            class="rounded-sm px-2 py-0.5 text-caption-sm font-medium transition-colors"
            :class="mode === 'block' ? 'bg-surface-elevated text-fg-primary shadow-xs' : 'text-fg-secondary'"
            @click="mode = 'block'"
          >
            {{ t("spaces.richtext.math.block") }}
          </button>
        </div>
      </div>

      <textarea
        v-model="latex"
        rows="3"
        class="w-full resize-none rounded-md border border-stroke-default bg-surface-canvas px-2.5 py-1.5 font-mono text-caption text-fg-primary outline-none focus-visible:border-stroke-focus focus-visible:ring-1 focus-visible:ring-stroke-focus"
        :placeholder="t('spaces.richtext.math.placeholder')"
        :aria-label="t('spaces.richtext.math.title')"
        @keydown.meta.enter.prevent="submit"
        @keydown.ctrl.enter.prevent="submit"
      />

      <div
        v-if="previewHtml"
        class="border-stroke-subtle bg-surface-subtle/60 max-h-28 overflow-auto rounded-md border px-2.5 py-2 text-center text-fg-primary"
        v-html="previewHtml"
      />

      <div class="flex items-center justify-between gap-2">
        <UiButton v-if="editingPos !== null" size="sm" variant="ghost" @click="remove">
          <Trash2 class="size-3.5 text-fg-danger" />
          {{ t("spaces.richtext.math.delete") }}
        </UiButton>
        <div class="flex-1" />
        <UiButton size="sm" variant="primary" :disabled="!latex.trim()" @click="submit">
          {{ editingPos === null ? t("spaces.richtext.math.insert") : t("spaces.richtext.math.apply") }}
        </UiButton>
      </div>
    </div>
  </UiPopover>
</template>
