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

const showsSummary = computed(
  () =>
    props.presentationContext.mode === "interactive" ||
    props.presentationContext.viewport.previewItemLimit >= 2,
);

const emptyMessage = computed(() =>
  props.data.metadataState === "failed" ? t("spaces.importExport.website.previewUnavailable") : undefined,
);
</script>

<template>
  <div class="flex size-full min-h-0 flex-col overflow-hidden bg-surface-panel select-none">
    <div
      class="flex h-9 shrink-0 items-center justify-between gap-2 border-b border-stroke-divider bg-surface-panel px-3"
    >
      <div class="flex min-w-0 flex-1 items-center gap-2 truncate">
        <div
          class="flex size-4 shrink-0 items-center justify-center overflow-hidden rounded-xs bg-surface-control"
        >
          <img
            v-if="faviconSource"
            :src="faviconSource"
            alt=""
            class="size-3.5 object-contain"
            draggable="false"
          />
          <Globe2 v-else class="size-2.5 text-fg-tertiary" :stroke-width="1.5" />
        </div>
        <div class="flex min-w-0 items-center gap-1 truncate text-caption-sm text-fg-tertiary">
          <span v-if="sourceName" class="truncate font-medium text-fg-secondary">{{ sourceName }}</span>
          <span v-if="sourceName" class="shrink-0 text-fg-tertiary">·</span>
          <span class="truncate text-fg-tertiary">{{ host }}</span>
        </div>
      </div>
      <SpaceCanvasNodeStatus :id="id" :data="data" />
    </div>

    <div class="flex min-h-0 flex-1 flex-col justify-between overflow-hidden px-3.5 py-2.5">
      <div class="flex min-h-0 flex-col gap-1.5 overflow-hidden">
        <strong
          class="line-clamp-2 shrink-0 text-label leading-snug font-semibold wrap-break-word text-fg-primary"
        >
          {{ data.title }}
        </strong>
        <p
          v-if="showsSummary && summary"
          class="line-clamp-3 shrink-0 text-caption leading-relaxed wrap-break-word text-fg-secondary"
        >
          {{ summary }}
        </p>
        <p v-else-if="emptyMessage" class="line-clamp-1 shrink-0 text-caption text-fg-tertiary">
          {{ emptyMessage }}
        </p>
      </div>

      <span class="shrink-0 truncate font-mono text-[10px] text-fg-tertiary">{{ host }}</span>
    </div>
  </div>
</template>
