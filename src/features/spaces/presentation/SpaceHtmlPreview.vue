<script setup lang="ts">
import { ShieldCheck } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { compact, take, uniq } from "es-toolkit";
import { hasProtocol } from "ufo";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps<{ source: string; previewPath: string; standalone: boolean; title: string }>();
const { t } = useI18n();
const frameSource = computed(() => convertFileSrc(props.previewPath));
const missingLocalAssets = computed(() => (props.standalone ? localAssetReferences(props.source).length : 0));

function localAssetReferences(source: string) {
  const document = new DOMParser().parseFromString(source, "text/html");
  const urls = [
    ...document.querySelectorAll("script[src], link[rel='stylesheet'][href], img[src], source[src]"),
  ].map((node) => node.getAttribute(node.hasAttribute("href") ? "href" : "src"));
  return take(uniq(compact(urls).filter(isLocalAssetReference)), 8);
}

function isLocalAssetReference(value: string) {
  const normalized = value.trim().toLowerCase();
  return !hasProtocol(normalized, { acceptRelative: true }) && !normalized.startsWith("#");
}
</script>

<template>
  <div class="relative h-64 overflow-hidden bg-surface-document">
    <iframe
      :title="title"
      :src="frameSource"
      sandbox="allow-scripts"
      loading="lazy"
      referrerpolicy="no-referrer"
      tabindex="-1"
      class="pointer-events-none block size-full border-0 bg-surface-document"
    />
    <div
      v-if="missingLocalAssets"
      class="pointer-events-none absolute inset-x-2 bottom-2 flex items-center gap-1.5 rounded-md bg-surface-overlay/90 px-2 py-1 text-caption-sm text-fg-secondary shadow-control backdrop-blur"
    >
      <ShieldCheck class="size-3 shrink-0 text-fg-tertiary" />
      <span class="min-w-0 truncate">{{
        t("spaces.canvas.htmlSandboxAssetsBlocked", { count: missingLocalAssets })
      }}</span>
    </div>
  </div>
</template>
