<script setup lang="ts">
import { CircleAlert } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSpinner from "../../../../../shared/ui/UiSpinner.vue";
import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import { useSpaceStore } from "../../../session/space-store";

const props = defineProps<{ id: string; data: SpaceNodePayload }>();
const { t } = useI18n();
const { mediaAnalysisIds } = storeToRefs(useSpaceStore());
const status = computed(() => {
  if (mediaAnalysisIds.value.includes(props.id)) return { label: t("spaces.media.analysisInProgress") };
  if (props.data.kind === "website") {
    if (props.data.metadataState === "loading") return { label: t("spaces.transfer.websiteLoading") };
    if (props.data.metadataState === "failed") {
      return { label: t("spaces.transfer.websitePreviewUnavailable"), failed: true };
    }
  }
  if (!("processingState" in props.data) || !props.data.processingState) return undefined;
  if (props.data.processingState === "failed") {
    return { label: t("spaces.processing.processingFailed"), failed: true };
  }
  return {
    label:
      props.data.processingState === "importing"
        ? t("spaces.transfer.importingItem")
        : t("spaces.transfer.importProcessing"),
  };
});
</script>

<template>
  <span v-if="status" :title="status.label" :aria-label="status.label" role="status">
    <CircleAlert v-if="status.failed" class="size-(--knovum-icon-size-xs) text-danger" :stroke-width="1.6" />
    <UiSpinner v-else size="sm" tone="faint" />
  </span>
</template>
