<script setup lang="ts">
import { Code2, PanelTop } from "@lucide/vue";
import { useToggle } from "@vueuse/core";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import type { CanvasNodePresentationContext } from "../../../canvas/contract/canvas-detail-profile";
import type { SpaceCodePayload } from "../../../model/nodes/node-payload";
import SpaceHtmlPreview from "../../SpaceHtmlPreview.vue";
import { projectPreviewItems } from "./canvas-preview-budget";

const props = defineProps<{
  data: SpaceCodePayload;
  presentationContext: CanvasNodePresentationContext;
  interactive: boolean;
}>();
const { t } = useI18n();
const rendered = shallowRef(false);
const toggleRendered = useToggle(rendered);
const htmlPreview = computed(() =>
  props.data.previewLanguage === "html" && props.data.previewPath
    ? { path: props.data.previewPath, source: props.data.previewText ?? "" }
    : undefined,
);

const source = computed(() =>
  projectPreviewItems((props.data.previewText ?? "").split(/\r?\n/u), props.presentationContext).join("\n"),
);

watch([() => props.presentationContext.mode, () => props.data.modifiedAt], ([mode]) => {
  if (mode !== "interactive") rendered.value = false;
});
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col overflow-hidden">
    <UiIconButton
      v-if="presentationContext.mode === 'interactive' && htmlPreview"
      class="absolute top-2 right-2 z-10"
      :class="interactive && 'pointer-events-auto'"
      :label="rendered ? t('spaces.inspector.showSource') : t('spaces.importExport.preview.show')"
      size="sm"
      @click.stop="toggleRendered()"
    >
      <Code2 v-if="rendered" class="size-(--knovum-icon-size-sm)" />
      <PanelTop v-else class="size-(--knovum-icon-size-sm)" />
    </UiIconButton>
    <SpaceHtmlPreview
      v-if="htmlPreview && rendered"
      :source="htmlPreview.source"
      :preview-path="htmlPreview.path"
      standalone
      :title="data.title"
      class="min-h-0 flex-1"
    />
    <pre
      v-else
      data-canvas-scrollable
      class="min-h-0 flex-1 p-3 font-mono text-caption leading-relaxed whitespace-pre-wrap text-foreground-secondary"
      :class="presentationContext.mode === 'interactive' ? 'overflow-auto' : 'overflow-hidden'"
      @wheel.stop
      >{{ source }}</pre>
  </div>
</template>
