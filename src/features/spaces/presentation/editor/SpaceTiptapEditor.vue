<script setup lang="ts">
import { ArrowLeft, Download, FileText, ImagePlus, StickyNote } from "@lucide/vue";
import { EditorContent } from "@tiptap/vue-3";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { exportSpaceMarkdown } from "../../assets/export";
import { useSpaceTiptapEditor } from "../../editor/tiptap-editor";
import { useSpaceTiptapImageDrop } from "../../editor/tiptap-image-drop";
import SpaceTiptapBubbleMenu from "./SpaceTiptapBubbleMenu.vue";
import SpaceTiptapOutline from "./SpaceTiptapOutline.vue";
import SpaceTiptapSearch from "./SpaceTiptapSearch.vue";
import SpaceTiptapToolbar from "./SpaceTiptapToolbar.vue";

const { t } = useI18n();
const state = useSpaceTiptapEditor();
const { activeHeadingId, characterCount, close, editorLabel, editorNode, editor, headings } = state;
const { persist, revealHeading, saveStatus, scrollRoot, syncActiveHeading, title, wordCount } = state;
const imageDragging = useSpaceTiptapImageDrop(editor, scrollRoot, () => t("editor.imageUploadFailed"));
async function exportMarkdown() {
  if (!editor.value) return;
  try {
    await persist();
    if (await exportSpaceMarkdown(title.value, editor.value.getMarkdown()))
      toast.success(t("spaces.transfer.exportComplete"));
  } catch (error) {
    toast.error(String(error));
  }
}
</script>

<template>
  <section v-if="editorNode" class="absolute inset-0 z-30 flex min-h-0 flex-col bg-canvas">
    <header
      class="flex h-(--knovum-toolbar-height) shrink-0 items-center gap-2 border-b border-divider bg-chrome px-3"
    >
      <UiIconButton :label="t('common.back')" size="sm" @click="close">
        <ArrowLeft class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
      <component
        :is="editorNode.data.kind === 'note' ? StickyNote : FileText"
        class="size-(--knovum-icon-size-sm) text-foreground-secondary"
        :stroke-width="1.55"
      />
      <span class="text-caption font-medium text-foreground-secondary">{{ editorLabel }}</span>
      <div class="flex-1" />
      <span class="text-micro font-medium text-foreground-tertiary">
        {{
          saveStatus === "saving"
            ? t("common.saving")
            : saveStatus === "unsaved"
              ? t("common.unsaved")
              : t("common.saved")
        }}
      </span>
      <SpaceTiptapSearch v-if="editor" :editor="editor" />
      <UiIconButton :label="t('spaces.transfer.exportMarkdown')" size="sm" @click="exportMarkdown">
        <Download class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
      <span class="hidden text-micro text-foreground-tertiary sm:block">
        {{ wordCount }} {{ t("editor.words") }} · {{ characterCount }} {{ t("editor.characters") }}
      </span>
    </header>
    <div v-if="editor" class="border-b border-divider bg-chrome/80">
      <SpaceTiptapToolbar :editor="editor" />
    </div>
    <div class="relative flex min-h-0 flex-1">
      <div ref="scrollRoot" class="relative min-h-0 flex-1 overflow-auto" @scroll.passive="syncActiveHeading">
        <div
          v-if="imageDragging"
          class="pointer-events-none sticky top-2 z-40 mx-2 -mb-18 flex h-16 items-center justify-center gap-2 rounded-panel border border-dashed border-accent bg-overlay/92 text-label font-medium text-accent shadow-popover backdrop-blur"
        >
          <ImagePlus class="size-(--knovum-icon-size-md)" />{{ t("editor.imageDropNow") }}
        </div>
        <article class="mx-auto min-h-full w-full max-w-3xl px-10 py-12 sm:px-14">
          <input
            v-model="title"
            class="w-full bg-transparent text-display font-semibold tracking-title text-foreground outline-none placeholder:text-foreground-tertiary"
            :aria-label="t('spaces.editor.markdownTitle')"
            :placeholder="
              editorNode.data.kind === 'note'
                ? t('spaces.editor.newNote')
                : t('spaces.editor.untitledMarkdown')
            "
            @blur="persist"
          />
          <SpaceTiptapBubbleMenu v-if="editor" :editor="editor" />
          <EditorContent v-if="editor" :editor="editor" class="mt-7" />
        </article>
      </div>
      <SpaceTiptapOutline :headings="headings" :active-id="activeHeadingId" @select="revealHeading" />
    </div>
  </section>
</template>

<style scoped>
@import "../../editor/space-tiptap-editor.css";
</style>
