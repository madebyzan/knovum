<script setup lang="ts">
import { Bold, Check, Italic, List, Underline } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { BubbleMenu } from "@tiptap/vue-3/menus";
import { computed, onBeforeUnmount, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import { canvasBaseFontSize, effectiveCanvasFontSize } from "../../../canvas/document/canvas-typography";
import { CANVAS_FONT_SIZES, CANVAS_INLINE_TEXT_COLORS } from "../canvas-typography-options";
import SpaceCanvasColorSwatches from "../SpaceCanvasColorSwatches.vue";
import SpaceCanvasTextAlignmentMenu from "./SpaceCanvasTextAlignmentMenu.vue";

const props = defineProps<{
  editor: Editor;
  defaultFontSize: number;
  defaultAlignment: "left" | "center" | "right";
  typographyScale: number;
}>();
const { t } = useI18n();
interface CanvasTextStyleAttributes {
  color?: string;
  fontSize?: string;
}

const textStyle = shallowRef<CanvasTextStyleAttributes>(props.editor.getAttributes("textStyle"));
const colorOptions = computed(() =>
  CANVAS_INLINE_TEXT_COLORS.map(([value, color, name]) => ({
    value,
    color,
    label: t(`spaces.canvas.selectionTextColors.${name}`),
  })),
);
const selectedColor = computed(() => {
  return textStyle.value.color ?? "default";
});
const selectedFontSize = computed(() => {
  const value = Number.parseFloat(textStyle.value.fontSize ?? "");
  return effectiveCanvasFontSize(
    Number.isFinite(value) ? value : props.defaultFontSize,
    props.typographyScale,
  );
});
const selectedAlignment = computed(() => {
  void textStyle.value;
  return (
    (["left", "center", "right"] as const).find((value) => props.editor.isActive({ textAlign: value })) ??
    props.defaultAlignment
  );
});

function refresh() {
  textStyle.value = props.editor.getAttributes("textStyle");
}

function setColor(value: string) {
  const chain = props.editor.chain().focus();
  if (value === "default") chain.unsetColor().run();
  else chain.setColor(value).run();
}

function setEffectiveFontSize(size: number) {
  props.editor
    .chain()
    .focus()
    .setFontSize(`${canvasBaseFontSize(size, props.typographyScale)}pt`)
    .run();
}

function shouldShow({ from, to }: { from: number; to: number }) {
  return from !== to;
}

function appendToCanvasOverlay() {
  return document.body;
}

onMounted(() => {
  props.editor.on("selectionUpdate", refresh);
  props.editor.on("transaction", refresh);
});
onBeforeUnmount(() => {
  props.editor.off("selectionUpdate", refresh);
  props.editor.off("transaction", refresh);
});
</script>

<template>
  <span class="contents" data-canvas-text-menu-host>
    <BubbleMenu
      :editor="editor"
      :append-to="appendToCanvasOverlay"
      :should-show="shouldShow"
      :update-delay="40"
    >
      <div
        class="flex items-center gap-0.5 rounded-control border border-border material-overlay p-0.5 shadow-popover"
      >
        <UiIconButton
          :label="t('editor.bold')"
          size="sm"
          :active="editor.isActive('bold')"
          @click="editor.chain().focus().toggleBold().run()"
        >
          <Bold class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
        </UiIconButton>
        <UiIconButton
          :label="t('editor.italic')"
          size="sm"
          :active="editor.isActive('italic')"
          @click="editor.chain().focus().toggleItalic().run()"
        >
          <Italic class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
        </UiIconButton>
        <UiIconButton
          :label="t('editor.underline')"
          size="sm"
          :active="editor.isActive('underline')"
          @click="editor.chain().focus().toggleUnderline().run()"
        >
          <Underline class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
        </UiIconButton>
        <UiIconButton
          :label="t('editor.bulletList')"
          size="sm"
          :active="editor.isActive('bulletList')"
          @click="editor.chain().focus().toggleBulletList().run()"
        >
          <List class="size-(--knovum-icon-size-sm)" :stroke-width="1.7" />
        </UiIconButton>
        <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
        <UiDropdownMenu side="top" align="center" :side-offset="5" :modal="false" class="min-w-20">
          <template #trigger>
            <button
              type="button"
              class="h-(--knovum-control-height-sm) min-w-10 rounded-control px-1.5 text-caption font-medium text-foreground-secondary tabular-nums transition-colors duration-100 outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring"
              :aria-label="t('spaces.canvas.fontSize')"
            >
              {{ selectedFontSize }}
            </button>
          </template>
          <UiDropdownMenuItem
            v-for="size in CANVAS_FONT_SIZES"
            :key="size"
            @select="setEffectiveFontSize(size)"
          >
            <Check
              class="size-(--knovum-icon-size-xs)"
              :class="selectedFontSize === size ? 'opacity-100' : 'opacity-0'"
            />
            <span class="tabular-nums">{{ size }} pt</span>
          </UiDropdownMenuItem>
        </UiDropdownMenu>
        <SpaceCanvasColorSwatches
          :label="t('spaces.canvas.textColor')"
          :options="colorOptions"
          :selected="selectedColor"
          :columns="6"
          @select="setColor"
        />
        <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
        <SpaceCanvasTextAlignmentMenu :editor="editor" :selected="selectedAlignment" />
      </div>
    </BubbleMenu>
  </span>
</template>
