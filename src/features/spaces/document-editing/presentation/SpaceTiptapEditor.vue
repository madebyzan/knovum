<script setup lang="ts">
import { EditorContent } from "@tiptap/vue-3";
import { shallowRef, toRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { useCitationHover } from "../../conversation/presentation/use-citation-hover";
import { useSpaceStore } from "../../session/space-store";
import type { SpaceEditorTab } from "../../workspace/workbench-tab";
import { createSpaceSlashCallbacks } from "../slash-commands";
import type { MathEditTarget } from "../tiptap-math";
import { convertSelectionToInlineMath } from "../tiptap-math-rules";
import { createSlashCommandController } from "../tiptap-slash-command";
import { createUntitledTitlePromoter } from "../tiptap-writing-text";
import { useSpaceTiptapEditor } from "../use-space-tiptap-editor";
import { useSpaceTiptapImageDrop } from "../use-space-tiptap-image-drop";
import { useSpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";
import SpaceDocumentSource from "./SpaceDocumentSource.vue";
import SpaceEditorProblem from "./SpaceEditorProblem.vue";
import SpaceTiptapCalloutView from "./SpaceTiptapCalloutView.vue";
import SpaceTiptapCodeBlockView from "./SpaceTiptapCodeBlockView.vue";
import SpaceTiptapHeader from "./SpaceTiptapHeader.vue";
import SpaceTiptapImageGroupView from "./SpaceTiptapImageGroupView.vue";
import SpaceTiptapImageView from "./SpaceTiptapImageView.vue";
import SpaceTiptapOverlays from "./SpaceTiptapOverlays.vue";
import SpaceTiptapSourceEditor from "./SpaceTiptapSourceEditor.vue";
import SpaceTiptapTitle from "./SpaceTiptapTitle.vue";
import SpaceTiptapToolbar from "./SpaceTiptapToolbar.vue";
import SpaceTiptapViewportOverlays from "./SpaceTiptapViewportOverlays.vue";
import { useSpaceTiptapSurface } from "./use-space-tiptap-surface";

const props = defineProps<{ tab: SpaceEditorTab; active: boolean }>();
const store = useSpaceStore();
const { t } = useI18n();
const activeRef = toRef(props, "active");
const scrollRoot = useTemplateRef<HTMLElement>("scrollRoot");
const articleHost = useTemplateRef<HTMLElement>("articleHost");
const proseHost = useTemplateRef<HTMLElement>("proseHost");
const titleField = useTemplateRef<InstanceType<typeof SpaceTiptapTitle>>("titleField");
const header = useTemplateRef<InstanceType<typeof SpaceTiptapHeader>>("header");
const viewportOverlays = useTemplateRef<InstanceType<typeof SpaceTiptapViewportOverlays>>("viewportOverlays");
const scrolled = shallowRef(false);
const mathTarget = shallowRef<MathEditTarget | null>(null);
const slashController = createSlashCommandController();
const openMath = (target: MathEditTarget) => (mathTarget.value = target);

const doc = useSpaceTiptapEditor(props.tab, activeRef, scrollRoot, {
  nodeViews: {
    codeBlock: SpaceTiptapCodeBlockView,
    callout: SpaceTiptapCalloutView,
    image: SpaceTiptapImageView,
    imageGroup: SpaceTiptapImageGroupView,
  },
  slashExtension: slashController.createExtension(),
  onOpenAi: () => {
    if (!surface.sourceMode.value) writing.openPrompt();
  },
  onFocusTitle: () => titleField.value?.focusEnd(),
  onEditMath: openMath,
  mathPlaceholders: { block: `Σ  ${t("spaces.richtext.math.emptyBlock")}`, inline: "TeX" },
});
const writing = useSpaceTiptapWritingTools(doc.editor, {
  documentTitle: doc.title,
  promoteTitle: createUntitledTitlePromoter(doc.title, () => titlePlaceholder.value),
});
const surface = useSpaceTiptapSurface({
  active: activeRef,
  doc,
  writing,
  onFind: () => header.value?.showSearch(),
  onFocusOutline: () => viewportOverlays.value?.focusOutline(),
});
const { showToolbar, fullWidth, sourceMode, sourceDraft, readOnly, titlePlaceholder } = surface;
slashController.attachCallbacks(createSpaceSlashCallbacks(() => doc.editor.value, writing, openMath));
const citationHover = useCitationHover(doc.groundedCitations);
const imageDrop = useSpaceTiptapImageDrop(doc.editor, scrollRoot, proseHost, activeRef);

function handleScroll() {
  scrolled.value = (scrollRoot.value?.scrollTop ?? 0) > 8;
  doc.syncActiveHeading();
}
</script>

<template>
  <section v-show="active" class="absolute inset-0 z-30 flex min-h-0 flex-col bg-surface-canvas">
    <SpaceTiptapHeader
      ref="header"
      :kind="doc.editorNode.value.data.kind"
      :editor-label="doc.editorLabel.value"
      :save-status="doc.saveStatus.value"
      :editor="doc.editor.value"
      :active="active"
      :show-toolbar="showToolbar"
      :full-width="fullWidth"
      :source-mode="sourceMode"
      :scrolled="scrolled"
      :word-count="doc.wordCount.value"
      :character-count="doc.characterCount.value"
      @open-ai="writing.openPrompt()"
      @ask-in-chat="surface.askInChat()"
      @toggle-toolbar="showToolbar = !showToolbar"
      @toggle-full-width="fullWidth = !fullWidth"
      @toggle-source-mode="surface.toggleSourceMode"
      @export-markdown="surface.exportMarkdown"
    />
    <SpaceEditorProblem
      v-if="doc.error.value"
      :error="doc.error.value"
      @discard="store.discardUnavailableEditorTab(tab.id)"
    />
    <div
      v-if="doc.editor.value && !readOnly && showToolbar && !sourceMode"
      class="relative z-20 border-b border-stroke-divider bg-surface-muted/65"
    >
      <SpaceTiptapToolbar :editor="doc.editor.value" />
    </div>
    <SpaceDocumentSource
      :active="active"
      :target="doc.sourceReadingTarget.value"
      @close="doc.closeSourceReading"
    >
      <div
        ref="scrollRoot"
        class="relative isolate z-0 min-h-0 flex-1 overflow-auto"
        @scroll.passive="handleScroll"
      >
        <article
          ref="articleHost"
          class="relative mx-auto min-h-full w-full px-12 pt-12 pb-44 transition-[max-width] duration-200 sm:px-20"
          :class="fullWidth ? 'max-w-5xl' : 'max-w-[780px]'"
        >
          <SpaceTiptapTitle
            ref="titleField"
            v-model="doc.title.value"
            :disabled="readOnly"
            :label="t('spaces.editor.markdown.title')"
            :placeholder="titlePlaceholder"
            @enter-body="doc.editor.value?.commands.focus('start')"
            @blur="surface.persistDocument"
          />
          <SpaceTiptapSourceEditor
            v-if="sourceMode"
            v-model="sourceDraft"
            :disabled="readOnly"
            @blur="surface.persistDocument"
          />
          <div v-else-if="doc.editor.value" ref="proseHost" class="relative mt-6">
            <EditorContent
              :editor="doc.editor.value"
              @pointerover="citationHover.onBodyPointerOver"
              @pointerout="citationHover.onBodyPointerOut"
            />
            <SpaceTiptapOverlays
              :editor="doc.editor.value"
              :writing="writing"
              :article-host="articleHost"
              :prose-host="proseHost"
              :scroll-root="scrollRoot"
              :math-target="mathTarget"
              :drop-active="imageDrop.active.value"
              :drop-indicator-top="imageDrop.dropIndicator.value?.lineTop ?? null"
              @ask-in-chat="(quote) => surface.askInChat(quote)"
              @close-math="mathTarget = null"
              @convert-to-math="doc.editor.value && convertSelectionToInlineMath(doc.editor.value, openMath)"
            />
          </div>
        </article>
      </div>
      <SpaceTiptapViewportOverlays
        ref="viewportOverlays"
        :editor="doc.editor.value"
        :node-id="doc.editorNode.value.id"
        :writing="writing"
        :full-width="fullWidth"
        :source-mode="sourceMode"
        :slash-controller="slashController"
        :citation-hover="citationHover"
        :headings="doc.headings.value"
        :active-heading-id="doc.activeHeadingId.value"
        @open-citation="doc.openGroundedCitation"
        @reveal-heading="doc.revealHeading"
      />
    </SpaceDocumentSource>
  </section>
</template>

<style scoped>
@import "../space-tiptap-editor.css";
</style>
