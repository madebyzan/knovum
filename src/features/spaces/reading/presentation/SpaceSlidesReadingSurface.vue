<script setup lang="ts">
import { ChevronLeft, ChevronRight, FileText, GalleryHorizontalEnd, PanelBottom, Rows2 } from "@lucide/vue";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSegmentedControl from "../../../../shared/ui/UiSegmentedControl.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceFilePayload } from "../../model/nodes/node-payload";
import { projectPresentationSlides } from "../slides-deck-projection";
import SpaceSlidesContinuousView from "./SpaceSlidesContinuousView.vue";
import SpaceSlidesFilmstrip from "./SpaceSlidesFilmstrip.vue";
import SpaceSlidesStage from "./SpaceSlidesStage.vue";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceFilePayload>;
  chunks: readonly KnowledgeChunk[];
  loading: boolean;
  recognizedText: (resourceId: string) => string | undefined;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
}>();

const { t } = useI18n();
const continuousView = useTemplateRef<InstanceType<typeof SpaceSlidesContinuousView>>("continuousView");
const activeSlideNumber = shallowRef(1);
const layoutMode = shallowRef<"single" | "continuous">("single");
const showFilmstrip = shallowRef(true);
const showNotes = shallowRef(true);

const slides = computed(() => projectPresentationSlides(props.node.data, props.chunks, props.recognizedText));
const totalSlides = computed(() => slides.value.length);
const activeIndex = computed(() => {
  const idx = slides.value.findIndex((slide) => slide.slideNumber === activeSlideNumber.value);
  return idx >= 0 ? idx : 0;
});
const currentSlide = computed(() => slides.value[activeIndex.value]);
const currentHasNotes = computed(() => {
  const slide = currentSlide.value;
  return Boolean(slide?.images.length && (slide.textBlocks.length || slide.ocrPassages.length));
});
const layoutOptions = computed(() => [
  { value: "single" as const, title: t("spaces.inspector.reader.singleSlide"), icon: GalleryHorizontalEnd },
  { value: "continuous" as const, title: t("spaces.inspector.reader.continuousSlides"), icon: Rows2 },
]);
const targetSlide = computed(() => {
  if (props.locator?.location.kind === "slides") return props.locator.location.slide;
  const needle = props.quote?.trim();
  if (!needle) return undefined;
  return slides.value.find(
    (s) => s.textBlocks.some((b) => b.text.includes(needle)) || s.ocrPassages.some((p) => p.includes(needle)),
  )?.slideNumber;
});

const NEXT_SLIDE_KEYS = new Set(["ArrowRight", "ArrowDown", "PageDown", " "]);
const PREV_SLIDE_KEYS = new Set(["ArrowLeft", "ArrowUp", "PageUp"]);

function selectIndex(index: number) {
  const slide = slides.value[Math.min(Math.max(0, index), slides.value.length - 1)];
  if (!slide) return;
  activeSlideNumber.value = slide.slideNumber;
  if (layoutMode.value === "continuous")
    void nextTick(() => continuousView.value?.scrollToSlide(slide.slideNumber));
}
function onKeyDown(event: KeyboardEvent) {
  if (layoutMode.value !== "single") return;
  const delta = NEXT_SLIDE_KEYS.has(event.key) ? 1 : PREV_SLIDE_KEYS.has(event.key) ? -1 : 0;
  if (!delta) return;
  event.preventDefault();
  selectIndex(activeIndex.value + delta);
}

watch(
  () => [props.node.id, targetSlide.value, totalSlides.value] as const,
  ([, slideNumber, count], prev) => {
    if (!count) return;
    if (slideNumber !== undefined) {
      activeSlideNumber.value = slideNumber;
      showNotes.value = true;
      return;
    }
    if (prev?.[0] !== props.node.id || !slides.value.some((s) => s.slideNumber === activeSlideNumber.value))
      activeSlideNumber.value = slides.value[0]!.slideNumber;
  },
  { immediate: true },
);
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none" tabindex="0" @keydown="onKeyDown">
    <div
      v-if="totalSlides"
      data-reader-ignore-selection
      class="flex h-9 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider bg-surface-panel px-2.5 select-none"
    >
      <div class="flex items-center gap-2">
        <UiSegmentedControl v-if="totalSlides > 1" v-model="layoutMode" :options="layoutOptions" icon-only />
        <span class="truncate text-caption font-medium text-fg-secondary tabular-nums">
          {{ t("spaces.inspector.pdfReader.position", { page: activeIndex + 1, total: totalSlides }) }}
        </span>
      </div>
      <div class="flex items-center gap-0.5">
        <UiIconButton
          v-if="layoutMode === 'single' && currentHasNotes"
          :label="t('spaces.inspector.extractedDocumentText')"
          size="sm"
          tooltip
          :class="showNotes && 'bg-surface-control text-fg-primary'"
          @click="showNotes = !showNotes"
        >
          <FileText class="size-3.5" />
        </UiIconButton>
        <UiIconButton
          v-if="layoutMode === 'single' && totalSlides > 1"
          :label="t('spaces.inspector.reader.thumbnails')"
          size="sm"
          tooltip
          :class="showFilmstrip && 'bg-surface-control text-fg-primary'"
          @click="showFilmstrip = !showFilmstrip"
        >
          <PanelBottom class="size-3.5" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.inspector.pdfReader.previousPage')"
          size="sm"
          tooltip
          :disabled="activeIndex <= 0"
          @click="selectIndex(activeIndex - 1)"
        >
          <ChevronLeft class="size-3.5" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.inspector.pdfReader.nextPage')"
          size="sm"
          tooltip
          :disabled="activeIndex >= totalSlides - 1"
          @click="selectIndex(activeIndex + 1)"
        >
          <ChevronRight class="size-3.5" />
        </UiIconButton>
      </div>
    </div>

    <template v-if="currentSlide && layoutMode === 'single'">
      <SpaceSlidesStage
        :slide="currentSlide"
        :can-previous="activeIndex > 0"
        :can-next="activeIndex < totalSlides - 1"
        :show-notes="showNotes"
        :has-notes="currentHasNotes"
        :exact-quote="currentSlide.slideNumber === targetSlide ? quote : undefined"
        @previous="selectIndex(activeIndex - 1)"
        @next="selectIndex(activeIndex + 1)"
      />
      <SpaceSlidesFilmstrip
        v-if="showFilmstrip && totalSlides > 1"
        :slides="slides"
        :active-index="activeIndex"
        @select="selectIndex"
      />
    </template>

    <SpaceSlidesContinuousView
      v-else-if="slides.length"
      ref="continuousView"
      :slides="slides"
      :target-slide="targetSlide"
      :quote="quote"
      @hover-slide="activeSlideNumber = $event"
    />
    <p
      v-else-if="!loading"
      data-reader-ignore-selection
      class="p-8 text-center text-caption text-fg-tertiary select-none"
    >
      {{ t("spaces.conversation.sourcePeek.noText") }}
    </p>
  </div>
</template>
