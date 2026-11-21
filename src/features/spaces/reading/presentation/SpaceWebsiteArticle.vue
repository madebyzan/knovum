<script setup lang="ts">
import { Globe } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, useTemplateRef, type CSSProperties } from "vue";
import { useI18n } from "vue-i18n";

import SpaceTextQuote from "../../knowledge/presentation/SpaceTextQuote.vue";
import type { SpaceWebsitePayload } from "../../model/nodes/node-payload";

const props = defineProps<{
  website: SpaceWebsitePayload;
  sections: readonly { heading?: string | undefined; text: string }[];
  activeIndex: number;
  quote?: string | undefined;
  proseStyle: CSSProperties;
}>();

const { t } = useI18n();
const scroller = useTemplateRef<HTMLDivElement>("scroller");
const faviconSrc = computed(() =>
  props.website.faviconPath ? convertFileSrc(props.website.faviconPath) : undefined,
);
const previewImageSrc = computed(() =>
  props.website.previewImagePath ? convertFileSrc(props.website.previewImagePath) : undefined,
);

defineExpose({
  scroller,
  scrollToSection: (index: string | number) => {
    scroller.value
      ?.querySelector<HTMLElement>(`[data-section-index="${index}"]`)
      ?.scrollIntoView({ behavior: "smooth", block: "center" });
  },
});
</script>

<template>
  <div ref="scroller" class="min-h-0 flex-1 overflow-y-auto bg-surface-raised px-7 py-6">
    <div class="mx-auto max-w-[68ch]">
      <header data-reader-ignore-selection class="border-b border-stroke-divider pb-4 select-none">
        <div class="flex min-w-0 items-center gap-2 text-caption font-medium text-fg-secondary">
          <img v-if="faviconSrc" :src="faviconSrc" alt="" class="size-4 shrink-0 rounded-xs object-contain" />
          <Globe v-else class="size-4 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
          <span class="truncate">{{ website.siteName || website.url }}</span>
        </div>
        <h1 class="mt-2.5 text-title leading-snug font-semibold break-words text-fg-primary">
          {{ website.pageTitle || website.title }}
        </h1>
        <img
          v-if="previewImageSrc"
          :src="previewImageSrc"
          alt=""
          class="mt-3.5 max-h-56 w-full rounded-lg border border-stroke-divider bg-surface-control object-cover"
        />
      </header>
      <div v-if="sections.length" class="reader-prose mt-5 space-y-5" :style="proseStyle">
        <article
          v-for="(section, index) in sections"
          :key="index"
          :data-website-section="section.heading ?? ''"
          :data-section-index="index"
          class="space-y-2 rounded-lg transition-colors"
          :class="index === activeIndex && '-mx-2 bg-surface-accent-subtle/35 p-2'"
        >
          <h2 v-if="section.heading">{{ section.heading }}</h2>
          <SpaceTextQuote
            data-website-section-body
            :text="section.text"
            :exact="index === activeIndex ? quote : undefined"
          />
        </article>
      </div>
      <p
        v-else
        data-reader-ignore-selection
        class="py-8 text-center text-caption text-fg-tertiary select-none"
      >
        {{ t("spaces.conversation.sourcePeek.noText") }}
      </p>
    </div>
  </div>
</template>

<style src="./reader-prose.css"></style>
