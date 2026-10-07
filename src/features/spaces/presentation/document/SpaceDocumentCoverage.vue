<script setup lang="ts">
import { countBy } from "es-toolkit";
import { computed, shallowRef, watch } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { DocumentCoverage } from "../../model/nodes/schema/document-coverage-fields";
import { projectDocumentResources } from "./document-resource-projection";
import SpaceDocumentResourceGrid from "./SpaceDocumentResourceGrid.vue";
import SpaceDocumentResourceViewer from "./SpaceDocumentResourceViewer.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const report = computed(() =>
  props.node.data.kind === "file" ? props.node.data.documentCoverage : undefined,
);
const stateCounts = computed(() => countBy(report.value?.occurrences ?? [], ({ state }) => state));
const completed = computed(() => stateCounts.value.completed ?? 0);
const unsupported = computed(() => stateCounts.value.unsupported ?? 0);
const resources = computed(() => (report.value ? projectDocumentResources(report.value) : []));
const selectedResourceId = shallowRef<string>();
const selectedResource = computed(() =>
  resources.value.find(({ resource }) => resource.id === selectedResourceId.value),
);
const viewerOpen = computed({
  get: () => selectedResourceId.value !== undefined,
  set: (open) => {
    if (!open) selectedResourceId.value = undefined;
  },
});
watch(
  () => props.node.id,
  () => {
    selectedResourceId.value = undefined;
  },
);
type DocumentWarning = DocumentCoverage["warnings"][number];
const warningMessageKeys: Record<DocumentWarning, string> = {
  office_vector_drawings_unsupported: "spaces.processing.document.warnings.officeVectorDrawingsUnsupported",
  spreadsheet_image_cell_anchors_unavailable:
    "spaces.processing.document.warnings.spreadsheetImageCellAnchorsUnavailable",
  spreadsheet_chart_rendering_unsupported:
    "spaces.processing.document.warnings.spreadsheetChartRenderingUnsupported",
  legacy_office_visual_occurrence_coverage_limited:
    "spaces.processing.document.warnings.legacyOfficeVisualOccurrenceCoverageLimited",
  legacy_spreadsheet_formulas_unavailable:
    "spaces.processing.document.warnings.legacySpreadsheetFormulasUnavailable",
  pdf_visual_coverage_limited: "spaces.processing.document.warnings.pdfVisualCoverageLimited",
};
</script>

<template>
  <div v-if="report" class="space-y-2 text-body text-fg-secondary">
    <p v-if="report.pageCount !== null">
      {{
        $t("spaces.processing.document.textCoverage", {
          count: report.pagesWithExtractedText.length,
          total: report.pageCount,
        })
      }}
    </p>
    <p v-if="report.pagesWithoutExtractableContent.length">
      {{
        $t("spaces.processing.document.unclassifiedPages", {
          count: report.pagesWithoutExtractableContent.length,
        })
      }}
    </p>
    <p v-if="report.pagesRequiringRecognition.length">
      {{
        $t("spaces.processing.document.recognitionRequired", {
          pages: report.pagesRequiringRecognition.join(", "),
        })
      }}
    </p>
    <p v-if="report.occurrences.length">
      {{
        $t("spaces.processing.document.visualCoverage", {
          count: completed,
          total: report.occurrences.length,
        })
      }}
    </p>
    <p v-if="unsupported">{{ $t("spaces.processing.document.visualUnavailable", { count: unsupported }) }}</p>
    <p v-for="warning in report.warnings" :key="warning">
      {{ $t(warningMessageKeys[warning]) }}
    </p>
    <section v-if="resources.length" class="space-y-2 pt-1">
      <header class="flex items-baseline justify-between gap-2">
        <strong class="text-label font-medium text-fg-primary">
          {{ $t("spaces.processing.document.resources.title") }}
        </strong>
        <span class="text-caption-sm text-fg-tertiary">{{ resources.length }}</span>
      </header>
      <SpaceDocumentResourceGrid
        :key="node.id"
        :resources="resources"
        :source-revision="report.sourceRevision"
        @select="selectedResourceId = $event.resource.id"
      />
    </section>
  </div>
  <SpaceDocumentResourceViewer
    v-if="report && selectedResource"
    v-model:open="viewerOpen"
    :item="selectedResource"
    :source-revision="report.sourceRevision"
  />
</template>
