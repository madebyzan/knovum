<script setup lang="ts">
import { Code2, PanelTop } from "@lucide/vue";
import { useToggle } from "@vueuse/core";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import type { SpaceCodePayload } from "../../../model/nodes/node-payload";
import SpaceHtmlPreview from "../../SpaceHtmlPreview.vue";

const props = defineProps<{
  data: SpaceCodePayload;
  detailLevel: "compact" | "summary" | "full";
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

watch([() => props.detailLevel, () => props.data.modifiedAt], ([detailLevel]) => {
  if (detailLevel !== "full") rendered.value = false;
});
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col overflow-hidden">
    <UiIconButton
      v-if="detailLevel === 'full' && htmlPreview"
      class="absolute top-2 right-2 z-10"
      :class="interactive && 'pointer-events-auto'"
      :label="rendered ? t('spaces.inspector.showSource') : t('spaces.transfer.showPreview')"
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
      class="min-h-0 flex-1 overflow-auto p-3 font-mono text-caption leading-relaxed whitespace-pre text-foreground-secondary"
      @wheel.stop
      >{{ data.previewText ?? "" }}</pre>
  </div>
</template>
