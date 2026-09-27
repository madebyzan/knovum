<script setup lang="ts">
import { Globe2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { spaceAssetSource } from "../../../assets/node-preview-source";
import type { CanvasNodePresentationContext } from "../../../canvas/preview/policy";
import type { SpaceWebsitePayload } from "../../../model/nodes/node-payload";
import { truncateUtf16 } from "../../../model/text";
import SpaceCanvasNodeStatus from "./SpaceCanvasNodeStatus.vue";

const props = defineProps<{
  id: string;
  data: SpaceWebsitePayload;
  presentationContext: CanvasNodePresentationContext;
}>();
const { t } = useI18n();
const host = computed(() => new URL(props.data.url).hostname.replace(/^www\./u, ""));
const sourceName = computed(() => {
  const name = props.data.siteName?.trim();
  return name && name !== props.data.title ? name : undefined;
});
const summary = computed(() => {
  const value = props.data.description || props.data.contentPreview;
  return value ? truncateUtf16(value.replace(/\s+/gu, " ").trim(), 600) : undefined;
});
const faviconSource = computed(() =>
  props.data.faviconPath ? spaceAssetSource(props.data.faviconPath) : undefined,
);
const previewSource = computed(() =>
  props.data.previewImagePath ? spaceAssetSource(props.data.previewImagePath) : undefined,
);
const showsSummary = computed(
  () =>
    props.presentationContext.mode === "interactive" ||
    props.presentationContext.viewport.previewItemLimit >= 2,
);
const showsPreview = computed(
  () =>
    props.presentationContext.mode === "interactive" ||
    props.presentationContext.viewport.previewItemLimit >= 4,
);
const emptyMessage = computed(() => {
  if (props.data.metadataState === "failed") return t("spaces.importExport.website.previewUnavailable");
  return undefined;
});
</script>

<template>
  <div class="flex size-full min-h-0 flex-col overflow-hidden">
    <div class="flex h-12 shrink-0 items-center gap-2.5 border-b border-divider px-3">
      <div
        class="flex size-7 shrink-0 items-center justify-center overflow-hidden rounded-control bg-control"
      >
        <img
          v-if="faviconSource"
          :src="faviconSource"
          alt=""
          class="size-5 object-contain"
          draggable="false"
        />
        <Globe2 v-else class="size-(--knovum-icon-size-sm) text-foreground-tertiary" :stroke-width="1.5" />
      </div>
      <div class="min-w-0 flex-1">
        <p v-if="sourceName" class="truncate text-caption font-medium text-foreground-secondary">
          {{ sourceName }}
        </p>
        <p class="truncate text-micro text-foreground-tertiary">{{ host }}</p>
      </div>
      <SpaceCanvasNodeStatus :id="id" :data="data" />
    </div>
    <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden px-3 py-3">
      <strong class="line-clamp-2 text-body leading-snug font-semibold text-foreground">{{
        data.title
      }}</strong>
      <p
        v-if="showsSummary && summary"
        class="text-caption leading-relaxed text-foreground-secondary"
        :class="previewSource ? 'line-clamp-3' : 'line-clamp-7'"
      >
        {{ summary }}
      </p>
      <p v-else-if="emptyMessage" class="text-caption text-foreground-tertiary">{{ emptyMessage }}</p>
    </div>
    <img
      v-if="showsPreview && previewSource"
      :src="previewSource"
      alt=""
      class="block h-1/4 w-full shrink-0 border-t border-divider object-cover object-center"
      draggable="false"
      loading="lazy"
      decoding="async"
    />
  </div>
</template>
