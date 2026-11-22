<script setup lang="ts">
import { BookOpenText, Globe } from "@lucide/vue";
import { computed, nextTick, onMounted, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiOutlineRail, { type UiOutlineItem } from "../../../../shared/ui/UiOutlineRail.vue";
import UiScrollEdgeFade from "../../../../shared/ui/UiScrollEdgeFade.vue";
import UiSegmentedControl from "../../../../shared/ui/UiSegmentedControl.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceWebsitePayload } from "../../model/nodes/node-payload";
import SpaceReaderProseToolbar from "./SpaceReaderProseToolbar.vue";
import SpaceReaderWebviewHost from "./SpaceReaderWebviewHost.vue";
import SpaceWebsiteArticle from "./SpaceWebsiteArticle.vue";
import { useDomBlockFind } from "./use-dom-block-find";
import { useReaderActiveAnchor } from "./use-reader-active-anchor";
import { useReaderTypography } from "./use-reader-typography";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceWebsitePayload>;
  locator?: KnowledgeLocator | undefined;
  locateNonce?: number | undefined;
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
const outlineItems = computed<readonly UiOutlineItem[]>(() => {
  const items = sections.value.flatMap((sec, index) =>
    sec.heading ? [{ id: String(index), label: sec.heading, level: 2 }] : [],
  );
  return items.length > 1 ? items : [];
});
const outlineAnchor = useReaderActiveAnchor({
  scroller: articleScroller,
  selector: "[data-section-index]",
  keyOf: (el) => el.dataset.sectionIndex,
  validIds: () => outlineItems.value.map((item) => item.id),
  sourceKey: () => [props.node.id, sections.value.length, viewMode.value],
  enabled: () => viewMode.value === "reader" && outlineItems.value.length > 0,
});

function jumpToSection(id: string | number) {
  outlineAnchor.scrollTo(String(id), article.value?.sectionElement(id));
}
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

async function locateActiveSection() {
  if (activeIndex.value < 0) return;
  viewMode.value = "reader";
  await nextTick();
  const section = article.value?.sectionElement(activeIndex.value);
  const mark = section?.querySelector<HTMLElement>("mark[data-citation-mark]");
  if (mark) {
    mark.scrollIntoView({ behavior: "smooth", block: "center" });
    return;
  }
  jumpToSection(activeIndex.value);
}

onMounted(() => void locateActiveSection());
watch(
  () => props.node.id,
  () => {
    viewMode.value = sections.value.length ? "reader" : "web";
    find.reset();
  },
  { immediate: true },
);
watch(
  () => [props.locator, props.quote, props.locateNonce, sections.value.length] as const,
  () => void locateActiveSection(),
  { flush: "post" },
);
</script>

<template>
  <div
    class="flex h-full min-h-0 flex-col bg-surface-canvas outline-none"
    tabindex="-1"
    @keydown="find.onKeyDown"
  >
    <SpaceReaderProseToolbar :show-controls="viewMode === 'reader'" :find="find" :typography="typography">
      <template #leading>
        <UiSegmentedControl v-model="viewMode" :options="modeOptions" />
      </template>
    </SpaceReaderProseToolbar>
    <div class="relative min-h-0 flex-1 overflow-hidden">
      <SpaceReaderWebviewHost
        class="absolute inset-0"
        :class="viewMode !== 'web' && 'pointer-events-none invisible'"
        :inert="viewMode !== 'web'"
        :active="viewMode === 'web'"
        :url="node.data.url"
      />
      <div
        class="absolute inset-0 flex flex-col overflow-hidden"
        :class="viewMode !== 'reader' && 'pointer-events-none invisible'"
        :inert="viewMode !== 'reader'"
      >
        <SpaceWebsiteArticle
          ref="article"
          :website="node.data"
          :sections="sections"
          :active-index="activeIndex"
          :quote="quote"
          :prose-style="typography.proseStyle.value"
        />
        <UiScrollEdgeFade surface="raised" />
        <UiOutlineRail
          :items="outlineItems"
          :active-id="outlineAnchor.activeId.value"
          :label="t('spaces.inspector.reader.outline')"
          @select="jumpToSection"
        />
      </div>
    </div>
  </div>
</template>
