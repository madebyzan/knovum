<script setup lang="ts">
import { ArrowUpRight, BookOpenText, Globe, Search } from "@lucide/vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed, nextTick, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiFindBar from "../../../../shared/ui/UiFindBar.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSegmentedControl from "../../../../shared/ui/UiSegmentedControl.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceWebsitePayload } from "../../model/nodes/node-payload";
import SpaceReaderFontControls from "./SpaceReaderFontControls.vue";
import SpaceReaderOutlineBar, { type ReaderOutlineAnchor } from "./SpaceReaderOutlineBar.vue";
import SpaceReaderWebviewHost from "./SpaceReaderWebviewHost.vue";
import SpaceWebsiteArticle from "./SpaceWebsiteArticle.vue";
import { useDomBlockFind } from "./use-dom-block-find";
import { useReaderTypography } from "./use-reader-typography";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceWebsitePayload>;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
}>();

const { t } = useI18n();
const typography = useReaderTypography();
const article = useTemplateRef<InstanceType<typeof SpaceWebsiteArticle>>("article");
const articleScroller = computed(() => article.value?.scroller ?? null);
const viewMode = shallowRef<"reader" | "web">("reader");
const find = useDomBlockFind(articleScroller, {
  enabled: () => viewMode.value === "reader",
  selector: "article h2, article p, article li, [data-website-section-body]",
});

const modeOptions = computed(() => [
  { value: "reader" as const, label: t("spaces.inspector.reader.readerMode"), icon: BookOpenText },
  { value: "web" as const, label: t("spaces.inspector.reader.liveWeb"), icon: Globe },
]);
const sections = computed(() => {
  const list = props.node.data.websiteSections;
  if (list?.length) return list;
  const fallback = props.node.data.contentPreview?.trim() || props.node.data.description?.trim();
  return fallback ? [{ heading: undefined, text: fallback }] : [];
});
const sectionAnchors = computed<readonly ReaderOutlineAnchor[]>(() =>
  sections.value.flatMap((sec, index) => (sec.heading ? [{ key: index, label: sec.heading }] : [])),
);
const activeIndex = computed(() => {
  const targetSection =
    props.locator?.location.kind === "website" ? props.locator.location.section : undefined;
  if (targetSection) {
    const byHeading = sections.value.findIndex((item) => item.heading === targetSection);
    if (byHeading >= 0) return byHeading;
  }
  const needle = props.quote?.trim();
  return needle ? sections.value.findIndex((item) => item.text.includes(needle)) : -1;
});

async function openOriginal() {
  try {
    await openUrl(props.node.data.url);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}

watch(
  () => props.node.id,
  () => {
    viewMode.value = sections.value.length ? "reader" : "web";
    find.reset();
  },
  { immediate: true },
);
watch(
  () => [props.locator, props.quote, sections.value.length] as const,
  async () => {
    if (activeIndex.value < 0) return;
    viewMode.value = "reader";
    await nextTick();
    article.value?.scrollToSection(activeIndex.value);
  },
  { immediate: true },
);
</script>

<template>
  <div
    class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none"
    tabindex="-1"
    @keydown="find.onKeyDown"
  >
    <div
      data-reader-ignore-selection
      class="flex h-9 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider bg-surface-panel px-2.5 select-none"
    >
      <UiSegmentedControl v-model="viewMode" :options="modeOptions" />
      <div class="flex items-center gap-1.5">
        <UiIconButton
          v-if="viewMode === 'reader'"
          :label="t('spaces.inspector.reader.searchPlaceholder')"
          size="sm"
          tooltip
          :class="find.open.value && 'bg-surface-control text-fg-primary'"
          @click="find.toggle()"
        >
          <Search class="size-3.5" />
        </UiIconButton>
        <SpaceReaderFontControls
          v-if="viewMode === 'reader'"
          :can-decrease="typography.canDecrease.value"
          :can-increase="typography.canIncrease.value"
          @decrease="typography.decrease()"
          @increase="typography.increase()"
        />
        <UiButton variant="secondary" size="sm" class="shrink-0 gap-1" @click="openOriginal">
          <span>{{ t("spaces.canvas.nodeActions.openOriginal") }}</span>
          <ArrowUpRight class="size-3.5" />
        </UiButton>
      </div>
    </div>

    <UiFindBar
      v-if="viewMode === 'reader' && find.open.value"
      v-model="find.query.value"
      :match-current="find.matched.value.length ? find.matchIndex.value + 1 : 0"
      :match-total="find.matched.value.length"
      @next="find.next()"
      @prev="find.prev()"
      @close="find.reset()"
    />
    <div class="relative min-h-0 flex-1 overflow-hidden">
      <SpaceReaderWebviewHost
        class="absolute inset-0"
        :class="viewMode === 'web' ? 'visible' : 'pointer-events-none invisible'"
        :inert="viewMode !== 'web'"
        :active="viewMode === 'web'"
        :url="node.data.url"
      />
      <div
        class="absolute inset-0 flex flex-col overflow-hidden"
        :class="viewMode === 'reader' ? 'visible' : 'pointer-events-none invisible'"
        :inert="viewMode !== 'reader'"
      >
        <div
          v-if="sectionAnchors.length > 1"
          data-reader-ignore-selection
          class="shrink-0 border-b border-stroke-divider bg-surface-panel/70 px-3 py-1.5 select-none"
        >
          <SpaceReaderOutlineBar :anchors="sectionAnchors" @select="article?.scrollToSection($event)" />
        </div>
        <SpaceWebsiteArticle
          ref="article"
          :website="node.data"
          :sections="sections"
          :active-index="activeIndex"
          :quote="quote"
          :prose-style="typography.proseStyle.value"
        />
      </div>
    </div>
  </div>
</template>
