<script setup lang="ts">
import { Download, FileText } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../../shared/ui/UiBadge.vue";
import UiDialog from "../../../../../shared/ui/UiDialog.vue";
import UiExpandableText from "../../../../../shared/ui/UiExpandableText.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import { knowledgeLocationLabelFromLocation } from "../../../knowledge/presentation/knowledge-location-label";
import type { ExportTarget } from "../../export/target";
import { useExportRun } from "../../export/use-export-run";
import { formatByteSize } from "../../format-byte-size";
import { spaceAssetSource } from "../../node-preview-source";
import SpaceExportMenu from "../export/SpaceExportMenu.vue";
import type { DocumentResourceProjection } from "./document-resource-projection";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  item: DocumentResourceProjection;
  sourceRevision: string;
  exportTarget: ExportTarget | undefined;
}>();
const { t } = useI18n();
const runExport = useExportRun();

function save(format: string) {
  if (props.exportTarget) void runExport(props.exportTarget, format);
}
const source = computed(() => spaceAssetSource(props.item.resource.path, props.sourceRevision));
const description = computed(
  () =>
    `${props.item.resource.width} × ${props.item.resource.height} · ${formatByteSize(props.item.resource.byteSize)}`,
);

function cleanContext(context: string, altText: string | null): string {
  const trimmedContext = context.trim();
  if (!altText) return trimmedContext;
  const trimmedAlt = altText.trim();
  if (trimmedContext === trimmedAlt) return "";
  if (trimmedContext.startsWith(trimmedAlt)) {
    return trimmedContext.slice(trimmedAlt.length).replace(/^[\r\n\s]+/, "");
  }
  return trimmedContext;
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="item.locationLabel"
    :description="description"
    size="wide"
    class="h-[min(680px,calc(100vh-5rem))] max-w-5xl"
    body-class="p-0 grid min-h-0 flex-1 md:grid-cols-[minmax(0,1fr)_340px] overflow-hidden"
  >
    <template #actions>
      <SpaceExportMenu :target="exportTarget" @save="save">
        <template #trigger>
          <UiIconButton
            :label="t('spaces.canvas.nodeActions.export')"
            size="sm"
            tooltip
            :disabled="!exportTarget"
          >
            <Download class="size-3.5" />
          </UiIconButton>
        </template>
      </SpaceExportMenu>
    </template>
    <div
      class="relative flex h-full min-h-0 items-center justify-center overflow-hidden bg-surface-canvas/50 p-6 select-none"
    >
      <img
        :src="source"
        :alt="item.locationLabel"
        class="max-h-full max-w-full object-contain select-none"
        draggable="false"
      />
    </div>

    <aside
      class="flex h-full min-h-0 flex-col overflow-y-auto border-l border-stroke-divider bg-surface-overlay p-4 text-caption text-fg-secondary"
    >
      <div class="flex items-center justify-between gap-2 border-b border-stroke-divider pb-3">
        <div>
          <span class="text-caption font-medium text-fg-primary">
            {{ t("spaces.processing.document.resources.occurrences", { count: item.occurrences.length }) }}
          </span>
          <p class="mt-0.5 text-caption-sm text-fg-tertiary">
            {{
              t(
                item.sourceKind === "embedded"
                  ? "spaces.processing.document.resources.embedded"
                  : "spaces.processing.document.resources.rendered",
              )
            }}
          </p>
        </div>
        <UiBadge size="sm">
          {{
            t(
              item.sourceKind === "embedded"
                ? "spaces.processing.document.resources.embeddedBadge"
                : "spaces.processing.document.resources.renderedBadge",
            )
          }}
        </UiBadge>
      </div>

      <div class="space-y-4 pt-3">
        <div
          v-for="(occurrence, index) in item.occurrences"
          :key="occurrence.id"
          class="space-y-3"
          :class="index > 0 ? 'border-t border-stroke-divider pt-3.5' : ''"
        >
          <div class="flex items-center gap-1.5 text-caption font-medium text-fg-primary">
            <FileText class="size-3.5 shrink-0 text-fg-tertiary" />
            <span
              class="min-w-0 flex-1 truncate"
              :title="knowledgeLocationLabelFromLocation(occurrence.location)"
            >
              {{ knowledgeLocationLabelFromLocation(occurrence.location) }}
            </span>
          </div>

          <div v-if="occurrence.altText" class="space-y-1">
            <span class="text-caption-sm font-medium text-fg-tertiary">
              {{ t("spaces.processing.document.resources.altText") }}
            </span>
            <p class="font-mono text-caption break-all text-fg-secondary select-text">
              {{ occurrence.altText }}
            </p>
          </div>

          <div v-if="cleanContext(occurrence.context, occurrence.altText)" class="space-y-1">
            <span class="text-caption-sm font-medium text-fg-tertiary">
              {{ t("spaces.processing.document.resources.context") }}
            </span>
            <UiExpandableText
              :text="cleanContext(occurrence.context, occurrence.altText)"
              :clamp-lines="3"
              preformatted
              class="text-caption leading-relaxed text-fg-secondary select-text"
            />
          </div>
        </div>
      </div>
    </aside>
  </UiDialog>
</template>
