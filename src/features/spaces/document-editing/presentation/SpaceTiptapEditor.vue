<script setup lang="ts">
import { ArrowLeft, Download, FileText, ImagePlus, StickyNote } from "@lucide/vue";
import { EditorContent } from "@tiptap/vue-3";
import { toRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { exportSpaceMarkdown } from "../../assets/export";
import { registerDocumentEditCommands } from "../../commands/document-edit-commands";
import { useSpaceStore } from "../../session/space-store";
import type { SpaceEditorTab } from "../../workspace/workbench-tab";
import { useSpaceTiptapEditor } from "../use-space-tiptap-editor";
import { useSpaceTiptapImageDrop } from "../use-space-tiptap-image-drop";
import { useSpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";
import SpaceDocumentOutline from "./SpaceDocumentOutline.vue";
import SpaceDocumentSource from "./SpaceDocumentSource.vue";
import SpaceEditorProblem from "./SpaceEditorProblem.vue";
import SpaceTiptapBubbleMenu from "./SpaceTiptapBubbleMenu.vue";
import SpaceTiptapSearch from "./SpaceTiptapSearch.vue";
import SpaceTiptapToolbar from "./SpaceTiptapToolbar.vue";

const props = defineProps<{ tab: SpaceEditorTab; active: boolean }>();
const store = useSpaceStore();
const { t } = useI18n();
const {
  activeHeadingId,
  characterCount,
  close,
  error,
  editorLabel,
  editorNode,
  editor,
  headings,
  persist,
  revealHeading,
  saveStatus,
  suspended,
  scrollRoot,
  syncActiveHeading,
  title,
  wordCount,
} = useSpaceTiptapEditor(props.tab, toRef(props, "active"));
const writing = useSpaceTiptapWritingTools(editor);
const imageDragging = useSpaceTiptapImageDrop(editor, scrollRoot, toRef(props, "active"));
const search = useTemplateRef<InstanceType<typeof SpaceTiptapSearch>>("search");
const outline = useTemplateRef<InstanceType<typeof SpaceDocumentOutline>>("outline");
watch(
  [editor, () => props.active, error, suspended],
  ([instance], _previous, cleanup) => {
    if (!instance || !props.active || error.value || suspended.value) return;
    cleanup(
      registerDocumentEditCommands({
        canUndo: () => instance.can().undo(),
        canRedo: () => instance.can().redo(),
        undo: () => {
          instance.commands.undo();
        },
        redo: () => {
          instance.commands.redo();
        },
        find: () => search.value?.show(),
        hasOutline: () => headings.value.length > 0,
        focusOutline: () => {
          void outline.value?.focus();
        },
        save: persist,
        close,
      }),
    );
  },
  { immediate: true },
);
async function exportMarkdown() {
  if (!editor.value) return;
  try {
    await persist();
    if (await exportSpaceMarkdown(title.value, editor.value.getMarkdown()))
      toast.success(t("spaces.importExport.export.completed"));
  } catch (caught) {
    toast.error(getErrorMessage(caught));
  }
}
</script>

<template>
  <section v-show="active" class="absolute inset-0 z-30 flex min-h-0 flex-col bg-surface-canvas">
    <header
      class="flex h-toolbar shrink-0 items-center gap-2 border-b border-stroke-divider bg-surface-chrome px-3"
    >
      <UiIconButton :label="t('common.back')" size="sm" @click="close">
        <ArrowLeft class="size-3.5" />
      </UiIconButton>
      <component
        :is="editorNode.data.kind === 'note' ? StickyNote : FileText"
        class="size-3.5 text-fg-secondary"
        :stroke-width="1.55"
      />
      <span class="text-caption font-medium text-fg-secondary">{{ editorLabel }}</span>
      <div class="flex-1" />
      <span class="text-caption-sm font-medium text-fg-tertiary">
        {{
          saveStatus === "saving"
            ? t("common.saving")
            : saveStatus === "unsaved"
              ? t("common.unsaved")
              : t("common.saved")
        }}
      </span>
      <SpaceTiptapSearch ref="search" v-if="editor && active" :editor="editor" />
      <UiIconButton :label="t('spaces.importExport.markdown.export')" size="sm" @click="exportMarkdown">
        <Download class="size-3.5" />
      </UiIconButton>
      <span class="hidden text-caption-sm text-fg-tertiary sm:block">
        {{ t("spaces.richtext.statistics.words", { count: wordCount }, wordCount) }} ·
        {{ t("spaces.richtext.statistics.characters", { count: characterCount }, characterCount) }}
      </span>
    </header>
    <SpaceEditorProblem v-if="error" :error="error" @discard="store.discardUnavailableEditorTab(tab.id)" />
    <div v-if="editor && !error && !suspended" class="border-b border-stroke-divider bg-surface-chrome/80">
      <SpaceTiptapToolbar :editor="editor" />
    </div>
    <div class="relative flex min-h-0 flex-1">
      <div ref="scrollRoot" class="relative min-h-0 flex-1 overflow-auto" @scroll.passive="syncActiveHeading">
        <div
          v-if="imageDragging"
          class="pointer-events-none sticky top-2 z-40 mx-2 -mb-18 flex h-16 items-center justify-center gap-2 rounded-panel border border-dashed border-stroke-accent bg-surface-overlay/92 text-label font-medium text-fg-accent shadow-popover backdrop-blur"
        >
          <ImagePlus class="size-4" />{{ t("spaces.richtext.image.dropNow") }}
        </div>
        <article class="mx-auto min-h-full w-full max-w-3xl px-10 py-12 sm:px-14">
          <input
            v-model="title"
            :disabled="!!error || suspended"
            class="w-full bg-transparent text-display font-semibold tracking-title text-fg-primary outline-none placeholder:text-fg-tertiary"
            :aria-label="t('spaces.editor.markdown.title')"
            :placeholder="
              editorNode.data.kind === 'note'
                ? t('spaces.editor.note.new')
                : t('spaces.editor.markdown.untitled')
            "
            @blur="persist"
          />
          <SpaceTiptapBubbleMenu
            v-if="editor && active && !error && !suspended"
            :editor="editor"
            :writing="writing"
          />
          <EditorContent v-if="editor" :editor="editor" class="mt-7" />
        </article>
      </div>
      <SpaceDocumentOutline
        ref="outline"
        :headings="headings"
        :active-id="activeHeadingId"
        @select="revealHeading"
      />
      <SpaceDocumentSource v-if="active" />
    </div>
  </section>
</template>

<style scoped>
@import "../space-tiptap-editor.css";
</style>
