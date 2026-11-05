<script setup lang="ts">
import { Globe2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { spaceAssetSource } from "../../../assets/node-preview-source";
import type { SpaceWebsitePayload } from "../../../model/nodes/node-payload";
import { truncateUtf16 } from "../../../model/text";
import type { CanvasNodePresentationContext } from "../../preview/policy";
import SpaceCanvasNodeStatus from "./SpaceCanvasNodeStatus.vue";
import { isRedundantWebsiteHost } from "./website-host";

const props = defineProps<{
  id: string;
  data: SpaceWebsitePayload;
  host: string;
  frame?: { width: number; height: number } | undefined;
  presentationContext: CanvasNodePresentationContext;
}>();
const { t } = useI18n();

const sourceName = computed(() => {
  const name = props.data.siteName?.trim();
  if (!name || name === props.data.title) return undefined;
  if (isRedundantWebsiteHost(name, props.host)) return undefined;
  return name;
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

const height = computed(() => props.frame?.height ?? 208);
const showsExcerpt = computed(() => showsSummary.value && Boolean(summary.value));

const emptyMessage = computed(() =>
  props.data.metadataState === "failed" ? t("spaces.importExport.website.previewUnavailable") : undefined,
);
</script>

<template>
  <div class="flex size-full min-h-0 flex-col overflow-hidden bg-surface-panel select-none">
    <div
      class="relative h-[54%] w-full shrink-0 overflow-hidden border-b border-stroke-divider bg-surface-control/20"
    >
      <img
        :src="previewSource"
        alt=""
        class="size-full object-cover object-center"
        draggable="false"
        loading="lazy"
        decoding="async"
      />
    </div>

    <div class="flex min-h-0 flex-1 flex-col justify-start gap-1 overflow-hidden bg-surface-panel px-3 py-2">
      <div class="flex h-4 shrink-0 items-center justify-between gap-1.5 text-caption-sm text-fg-tertiary">
        <div class="flex min-w-0 flex-1 items-center gap-1.5 truncate">
          <div
            class="flex size-3.5 shrink-0 items-center justify-center overflow-hidden rounded-xs bg-surface-control"
          >
            <img
              v-if="faviconSource"
              :src="faviconSource"
              alt=""
              class="size-3 object-contain"
              draggable="false"
            />
            <Globe2 v-else class="size-2.5 text-fg-tertiary" :stroke-width="1.5" />
          </div>
          <span v-if="sourceName" class="truncate font-medium text-fg-secondary">{{ sourceName }}</span>
          <span v-if="sourceName" class="shrink-0 text-fg-tertiary">·</span>
          <span class="truncate text-fg-tertiary">{{ host }}</span>
        </div>
        <SpaceCanvasNodeStatus :id="id" :data="data" />
      </div>

      <div class="flex min-h-0 flex-col gap-0.5 overflow-hidden">
        <strong
          :class="showsExcerpt ? 'line-clamp-1' : 'line-clamp-2'"
          class="shrink-0 text-label leading-snug font-semibold wrap-break-word text-fg-primary"
        >
          {{ data.title }}
        </strong>
        <p
          v-if="showsExcerpt"
          :class="height >= 195 ? 'line-clamp-2' : 'line-clamp-1'"
          class="shrink-0 text-caption-sm leading-normal wrap-break-word text-fg-secondary"
        >
          {{ summary }}
        </p>
        <p v-else-if="emptyMessage" class="line-clamp-1 shrink-0 text-caption-sm text-fg-tertiary">
          {{ emptyMessage }}
        </p>
      </div>
    </div>
  </div>
</template>
