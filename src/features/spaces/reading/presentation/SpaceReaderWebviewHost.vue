<script setup lang="ts">
import { ArrowLeft, ArrowRight, RotateCw } from "@lucide/vue";
import { useEventListener, useResizeObserver } from "@vueuse/core";
import { computed, inject, onBeforeUnmount, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { overlayScopeVisibleKey } from "../../../../shared/contracts/overlay-scope-visibility";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import {
  closeReaderWebview,
  navigateReaderWebview,
  openReaderAssetPreview,
  openReaderWebview,
  updateReaderWebviewBounds,
  type ReaderWebviewBounds,
  type ReaderWebviewNavigation,
} from "../reader-webview-operations";

interface ReaderAssetPreviewSource {
  spaceId: string;
  nodeId: string;
  revision?: string | undefined;
}

const props = withDefaults(
  defineProps<{
    url?: string | undefined;
    asset?: ReaderAssetPreviewSource | undefined;
    active?: boolean;
    showAddressBar?: boolean;
  }>(),
  { active: true, showAddressBar: true },
);

const { t } = useI18n();
const workspaceVisible = inject(overlayScopeVisibleKey, undefined);
const slotRef = useTemplateRef<HTMLDivElement>("slot");
const failure = shallowRef<string>();
const opened = shallowRef(false);
let frameId: number | undefined;
let syncToken = 0;
let lastLayoutSignature = "";
let loadedSourceKey = "";

const isVisible = computed(() => props.active && (workspaceVisible?.value ?? true));
const sourceKey = computed(() =>
  props.asset
    ? `asset:${props.asset.spaceId}:${props.asset.nodeId}:${props.asset.revision ?? ""}`
    : `url:${props.url ?? ""}`,
);

function readSlotBounds(): ReaderWebviewBounds | undefined {
  const rect = slotRef.value?.getBoundingClientRect();
  if (!rect || rect.width < 20 || rect.height < 20) return undefined;
  return {
    x: Math.round(rect.left),
    y: Math.round(rect.top),
    width: Math.round(rect.width),
    height: Math.round(rect.height),
  };
}

function layoutSignature(bounds: ReaderWebviewBounds): string {
  return `1:${bounds.x},${bounds.y},${bounds.width},${bounds.height}`;
}

function openTarget(bounds: ReaderWebviewBounds): Promise<void> {
  if (props.asset) return openReaderAssetPreview(props.asset.spaceId, props.asset.nodeId, bounds);
  if (props.url) return openReaderWebview(props.url, bounds);
  return Promise.resolve();
}

async function hideWebviewIfNeeded() {
  syncToken += 1;
  if (!opened.value || lastLayoutSignature === "0") return;
  lastLayoutSignature = "0";
  await updateReaderWebviewBounds({ x: 0, y: 0, width: 1, height: 1 }, false).catch(() => {});
}

async function openAndTrack(bounds: ReaderWebviewBounds, key: string, signature: string, token: number) {
  await openTarget(bounds);
  opened.value = true;
  loadedSourceKey = key;
  if (token !== syncToken || !isVisible.value) {
    if (!isVisible.value) await hideWebviewIfNeeded();
    return;
  }
  lastLayoutSignature = signature;
  failure.value = undefined;
}

async function syncWebview() {
  const token = ++syncToken;
  const bounds = isVisible.value ? readSlotBounds() : undefined;
  if (!bounds) {
    await hideWebviewIfNeeded();
    return;
  }
  const key = sourceKey.value;
  const signature = layoutSignature(bounds);
  try {
    if (!opened.value || loadedSourceKey !== key) {
      await openAndTrack(bounds, key, signature, token);
      return;
    }
    if (signature === lastLayoutSignature) return;
    lastLayoutSignature = signature;
    await updateReaderWebviewBounds(bounds, true);
  } catch (error) {
    if (token === syncToken) failure.value = getErrorMessage(error);
  }
}

function scheduleBoundsSync() {
  if (!isVisible.value) return;
  if (frameId !== undefined) cancelAnimationFrame(frameId);
  frameId = requestAnimationFrame(() => {
    frameId = undefined;
    void syncWebview();
  });
}

async function navigate(action: ReaderWebviewNavigation) {
  try {
    await navigateReaderWebview(action);
  } catch (error) {
    failure.value = getErrorMessage(error);
  }
}

useResizeObserver(slotRef, scheduleBoundsSync);
useResizeObserver(() => document.body, scheduleBoundsSync);
useEventListener(globalThis, "resize", scheduleBoundsSync);

watch(
  [slotRef, sourceKey, isVisible],
  ([slot, key, visible]) => {
    if (!visible) {
      if (loadedSourceKey && loadedSourceKey !== key) loadedSourceKey = "";
      void hideWebviewIfNeeded();
      return;
    }
    if (slot) void syncWebview();
  },
  { immediate: true, flush: "post" },
);

onBeforeUnmount(() => {
  if (frameId !== undefined) cancelAnimationFrame(frameId);
  syncToken += 1;
  opened.value = false;
  loadedSourceKey = "";
  void closeReaderWebview().catch(() => {});
});
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col bg-surface-canvas">
    <div
      v-if="showAddressBar && url"
      data-reader-ignore-selection
      class="flex shrink-0 items-center gap-1 border-b border-stroke-divider bg-surface-panel px-2.5 py-1 select-none"
    >
      <UiIconButton :label="t('spaces.space.back')" size="sm" tooltip @click="navigate('back')">
        <ArrowLeft class="size-3.5" />
      </UiIconButton>
      <UiIconButton :label="t('spaces.space.forward')" size="sm" tooltip @click="navigate('forward')">
        <ArrowRight class="size-3.5" />
      </UiIconButton>
      <UiIconButton :label="t('common.refresh')" size="sm" tooltip @click="navigate('reload')">
        <RotateCw class="size-3.5" />
      </UiIconButton>
      <span
        class="ml-1 min-w-0 flex-1 truncate rounded-md border border-stroke-divider/70 bg-surface-control px-2.5 py-0.5 text-caption-sm text-fg-secondary"
        :title="url"
      >
        {{ url }}
      </span>
    </div>
    <div ref="slot" class="relative min-h-0 flex-1 bg-surface-raised">
      <p v-if="failure" class="p-6 text-center text-caption text-fg-danger" role="alert">{{ failure }}</p>
    </div>
  </div>
</template>
