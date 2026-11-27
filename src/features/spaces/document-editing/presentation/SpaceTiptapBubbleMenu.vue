<script setup lang="ts">
import { MessageSquareQuote } from "@lucide/vue";
import type { Editor as CoreEditor } from "@tiptap/core";
import type { Editor } from "@tiptap/vue-3";
import { BubbleMenu } from "@tiptap/vue-3/menus";
import { useI18n } from "vue-i18n";

import { POPOVER_SURFACE_CLASSES } from "../../../../shared/ui/ui-classes";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import { InlineMathIcon } from "../editor-icons";
import type { SpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";
import SpaceTiptapInlineMarks from "./SpaceTiptapInlineMarks.vue";
import SpaceTiptapLinkControl from "./SpaceTiptapLinkControl.vue";
import SpaceTiptapTextStyleMenu from "./SpaceTiptapTextStyleMenu.vue";

const props = defineProps<{ editor: Editor; writing: SpaceTiptapWritingTools; mathOpen?: boolean }>();
const emit = defineEmits<{ askInChat: [text: string]; convertToMath: [] }>();
const { t } = useI18n();

function shouldShowSelectionMenu({ editor }: { editor: CoreEditor }) {
  if (editor.isDestroyed || !editor.isEditable || props.writing.phase.value !== "closed" || props.mathOpen)
    return false;
  const { from, to } = editor.state.selection;
  if (from === to || editor.isActive("codeBlock")) return false;
  return editor.state.doc.textBetween(from, to, " ").trim().length > 0;
}

function sendSelectionToChat() {
  const { from, to } = props.editor.state.selection;
  const text = props.editor.state.doc.textBetween(from, to, "\n", "\n").trim();
  if (text) emit("askInChat", text);
}
</script>

<template>
  <BubbleMenu :editor="editor" plugin-key="selectionBubbleMenu" :should-show="shouldShowSelectionMenu">
    <div :class="[POPOVER_SURFACE_CLASSES, 'z-50 flex items-center gap-0.5 p-1']">
      <button
        type="button"
        data-testid="bubble-ask-ai"
        :disabled="Boolean(props.writing.runningAction.value)"
        class="inline-flex h-(--knovum-control-height-sm) items-center gap-1.5 rounded-md px-2 text-label font-medium text-fg-primary transition-colors duration-100 ease-standard outline-none hover:bg-surface-control-hover disabled:opacity-40"
        @click="props.writing.openPrompt()"
      >
        <UiKnovumAiIcon class="size-3.5 text-fg-secondary" />
        <span class="whitespace-nowrap">{{ t("spaces.richtext.ai.panel.ask") }}</span>
      </button>
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <SpaceTiptapTextStyleMenu :editor="props.editor" />
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <SpaceTiptapInlineMarks :editor="props.editor" />
      <UiIconButton
        data-testid="bubble-inline-math"
        :label="t('spaces.richtext.math.convertAction')"
        tooltip
        size="sm"
        @click="emit('convertToMath')"
      >
        <InlineMathIcon class="size-3.5" :stroke-width="1.65" />
      </UiIconButton>
      <SpaceTiptapLinkControl :editor="props.editor" />
      <UiSeparator orientation="vertical" class="mx-0.5 !h-4" />
      <UiIconButton
        :label="t('spaces.richtext.blockMenu.askInChat')"
        tooltip
        size="sm"
        @click="sendSelectionToChat"
      >
        <MessageSquareQuote class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
    </div>
  </BubbleMenu>
</template>
