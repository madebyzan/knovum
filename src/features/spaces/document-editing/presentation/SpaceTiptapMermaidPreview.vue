<script setup lang="ts">
import { AlertCircle } from "@lucide/vue";
import { useDebounceFn } from "@vueuse/core";
import { computed, onBeforeUnmount, onMounted, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { renderMermaidDiagram } from "../../../../shared/ui/mermaid-renderer";
import { createMermaidStage, type MermaidStageHandle } from "../../../../shared/ui/mermaid-stage";
import type { MermaidViewerLabels } from "../../../../shared/ui/mermaid-viewport";

const props = defineProps<{ code: string }>();
const emit = defineEmits<{ edit: [] }>();
const { t } = useI18n();
const containerRef = useTemplateRef<HTMLElement>("containerRef");
const errorMessage = shallowRef<string | null>(null);
const hasSvg = shallowRef(false);
let renderNonce = 0;
let themeObserver: MutationObserver | undefined;
let stage: MermaidStageHandle | undefined;

const viewerLabels = computed<MermaidViewerLabels>(() => ({
  zoomIn: t("spaces.richtext.image.zoomIn"),
  zoomOut: t("spaces.richtext.image.zoomOut"),
  preview: t("spaces.richtext.image.preview"),
  close: t("common.close"),
}));

function isDarkTheme(): boolean {
  return typeof document !== "undefined" && document.documentElement.dataset.theme === "dark";
}

function ensureStage(): MermaidStageHandle | undefined {
  if (stage) return stage;
  const host = containerRef.value;
  if (!host) return undefined;
  stage = createMermaidStage(host, {
    showExpandButton: false,
    labels: viewerLabels.value,
    onEdit: () => emit("edit"),
  });
  return stage;
}

function mountSvg(svgMarkup: string, dark: boolean) {
  const handle = ensureStage();
  if (!handle) return;
  if (!svgMarkup) {
    handle.update("", dark);
    hasSvg.value = false;
    return;
  }
  hasSvg.value = true;
  if (!handle.update(svgMarkup, dark)) hasSvg.value = false;
}

const renderNow = useDebounceFn(async () => {
  const current = ++renderNonce;
  const source = props.code.trim();
  const dark = isDarkTheme();
  if (!source) {
    errorMessage.value = null;
    mountSvg("", dark);
    return;
  }
  const rendered = await renderMermaidDiagram(source, dark);
  if (current !== renderNonce) return;
  if (rendered.ok) {
    errorMessage.value = null;
    mountSvg(rendered.svg, dark);
  } else {
    errorMessage.value = rendered.error;
  }
}, 200);

watch(viewerLabels, (next) => stage?.setLabels(next));
watch(
  () => props.code,
  () => void renderNow(),
  { immediate: true },
);

onMounted(() => {
  ensureStage();
  if (typeof MutationObserver === "undefined") return;
  themeObserver = new MutationObserver(() => void renderNow());
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme"],
  });
});

onBeforeUnmount(() => {
  renderNow.cancel();
  themeObserver?.disconnect();
  stage?.destroy();
  stage = undefined;
});

defineExpose({
  openFullscreen: () => stage?.openFullscreen(),
});
</script>

<template>
  <div
    contenteditable="false"
    data-testid="code-block-mermaid-preview"
    class="relative flex min-h-24 flex-col items-center justify-center bg-surface-canvas"
  >
    <div ref="containerRef" class="w-full" :class="!hasSvg && 'hidden'" />
    <p v-if="!code.trim()" class="px-6 py-5 text-caption text-fg-tertiary">
      {{ t("spaces.richtext.mermaid.empty") }}
    </p>
    <div
      v-if="errorMessage"
      role="alert"
      class="mx-4 my-3 flex max-w-full items-start gap-2 rounded-md border border-stroke-danger/35 bg-surface-danger-subtle/60 px-3 py-2 text-caption text-fg-danger"
    >
      <AlertCircle class="mt-0.5 size-3.5 shrink-0" :stroke-width="1.6" />
      <span class="break-words">{{ t("spaces.richtext.mermaid.syntaxError") }}: {{ errorMessage }}</span>
    </div>
  </div>
</template>
