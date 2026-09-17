<script setup lang="ts">
import { computed } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

const props = defineProps<{ node: SpaceCanvasNode }>();
const report = computed(() =>
  props.node.data.kind === "file" ? props.node.data.documentCoverage : undefined,
);
const completed = computed(
  () => report.value?.occurrences.filter(({ state }) => state === "completed").length ?? 0,
);
const unsupported = computed(
  () => report.value?.occurrences.filter(({ state }) => state === "unsupported").length ?? 0,
);
</script>

<template>
  <div v-if="report" class="space-y-2 text-body text-foreground-secondary">
    <p v-if="report.pageCount !== null">
      {{
        $t("spaces.processing.documentTextCoverage", {
          count: report.pagesWithExtractedText.length,
          total: report.pageCount,
        })
      }}
    </p>
    <p v-if="report.pagesWithoutExtractableContent.length">
      {{
        $t("spaces.processing.documentUnclassifiedPages", {
          count: report.pagesWithoutExtractableContent.length,
        })
      }}
    </p>
    <p v-if="report.pagesRequiringRecognition.length">
      {{
        $t("spaces.processing.documentRecognitionRequired", {
          pages: report.pagesRequiringRecognition.join(", "),
        })
      }}
    </p>
    <p v-if="report.occurrences.length">
      {{
        $t("spaces.processing.documentVisualCoverage", { count: completed, total: report.occurrences.length })
      }}
    </p>
    <p v-if="unsupported">{{ $t("spaces.processing.documentVisualUnavailable", { count: unsupported }) }}</p>
    <p v-for="warning in report.warnings" :key="warning">
      {{ $t(`spaces.processing.documentWarnings.${warning}`) }}
    </p>
  </div>
</template>
