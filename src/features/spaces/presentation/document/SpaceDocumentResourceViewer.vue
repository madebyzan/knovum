<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiDialog from "../../../../shared/ui/UiDialog.vue";
import { formatByteSize } from "../../assets/format-byte-size";
import { spaceAssetSource } from "../../assets/node-preview-source";
import { knowledgeLocationLabelFromLocation } from "../../knowledge/presentation/location-label";
import type { DocumentResourceProjection } from "./document-resource-projection";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  item: DocumentResourceProjection;
  sourceRevision: string;
}>();
const { t } = useI18n();
const source = computed(() => spaceAssetSource(props.item.resource.path, props.sourceRevision));
const description = computed(
  () =>
    `${props.item.resource.width} × ${props.item.resource.height} · ${formatByteSize(props.item.resource.byteSize)}`,
);
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="item.locationLabel"
    :description="description"
    size="wide"
    body-class="grid min-h-0 gap-4 md:grid-cols-[minmax(0,1fr)_240px]"
  >
    <div class="grid min-h-80 place-items-center overflow-hidden rounded-md bg-surface-canvas p-3">
      <img
        :src="source"
        :alt="item.locationLabel"
        class="max-h-[60vh] max-w-full object-contain"
        draggable="false"
      />
    </div>
    <aside class="min-h-0 space-y-3 overflow-y-auto text-caption text-fg-secondary">
      <div>
        <strong class="text-label font-medium text-fg-primary">
          {{ t("spaces.processing.document.resources.occurrences", { count: item.occurrences.length }) }}
        </strong>
        <p class="mt-1 text-caption-sm text-fg-tertiary">
          {{
            t(
              item.sourceKind === "embedded"
                ? "spaces.processing.document.resources.embedded"
                : "spaces.processing.document.resources.rendered",
            )
          }}
        </p>
      </div>
      <article
        v-for="occurrence in item.occurrences"
        :key="occurrence.id"
        class="border-t border-stroke-divider pt-3 first:border-t-0 first:pt-0"
      >
        <strong class="text-caption font-medium text-fg-primary">
          {{ knowledgeLocationLabelFromLocation(occurrence.location) }}
        </strong>
        <p v-if="occurrence.altText" class="mt-1 whitespace-pre-wrap">{{ occurrence.altText }}</p>
        <p v-if="occurrence.context" class="mt-1 whitespace-pre-wrap text-fg-tertiary">
          {{ occurrence.context }}
        </p>
        <p v-if="occurrence.analysis?.description" class="mt-2 whitespace-pre-wrap">
          {{ occurrence.analysis.description }}
        </p>
        <div v-if="occurrence.analysis?.detectedText" class="mt-2">
          <strong class="text-caption-sm font-medium text-fg-tertiary">
            {{ t("spaces.processing.document.resources.detectedText") }}
          </strong>
          <p class="mt-1 whitespace-pre-wrap">{{ occurrence.analysis.detectedText }}</p>
        </div>
      </article>
    </aside>
  </UiDialog>
</template>
