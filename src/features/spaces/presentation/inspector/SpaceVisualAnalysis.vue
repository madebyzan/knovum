<script setup lang="ts">
import { RefreshCw } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import { retryEnrichmentJob } from "../../enrichment/enrichment-jobs";
import { useNodeEnrichmentStatus } from "../../enrichment/use-node-enrichment-status";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const { jobs, loading, refresh } = useNodeEnrichmentStatus(activeSpaceId, computed(() => props.node.id));
const visualJob = computed(() => jobs.value.find((job) => job.jobType === "analyze_visual"));
const retrying = shallowRef(false);
const knowledge = computed(() => {
  const data = props.node.data;
  return data.kind === "image" || data.kind === "video" ? data.visualKnowledge : undefined;
});

async function retry() {
  if (!visualJob.value || retrying.value) return;
  retrying.value = true;
  try {
    await retryEnrichmentJob(visualJob.value.id);
    await refresh();
  } catch (error) {
    toast.error(String(error));
  } finally {
    retrying.value = false;
  }
}
</script>

<template>
  <div>
    <template v-if="knowledge">
      <p class="text-body leading-relaxed text-fg-secondary select-text">
        {{ knowledge.description }}
      </p>
      <div v-if="knowledge.detectedText" class="mt-3">
        <h4 class="text-caption font-medium text-fg-tertiary">
          {{ t("spaces.inspector.visualDetectedText") }}
        </h4>
        <pre
          class="mt-1 font-sans text-caption leading-relaxed whitespace-pre-wrap text-fg-secondary select-text"
          >{{ knowledge.detectedText }}</pre>
      </div>
      <div v-if="knowledge.tags.length" class="mt-3 flex flex-wrap gap-1">
        <UiBadge v-for="tag in knowledge.tags" :key="tag" size="sm">
          {{ tag }}
        </UiBadge>
      </div>
    </template>
    <p
      v-else-if="loading || visualJob?.status === 'pending' || visualJob?.status === 'running'"
      class="text-caption leading-relaxed text-fg-tertiary"
      role="status"
    >
      {{ t("spaces.knowledge.visual.pending") }}
    </p>
    <p
      v-else-if="visualJob?.status === 'awaiting_configuration'"
      class="text-caption leading-relaxed text-fg-tertiary"
    >
      {{ t("spaces.processing.visual.waitingConfiguration") }}
    </p>
    <div
      v-else-if="visualJob?.status === 'failed'"
      class="flex items-center justify-between gap-2 text-caption text-fg-danger"
    >
      <span>{{ t("spaces.processing.failed") }}</span>
      <UiButton size="sm" variant="ghost" :disabled="retrying" @click="retry">
        <RefreshCw class="size-3" :class="retrying && 'animate-spin'" />{{ t("common.retry") }}
      </UiButton>
    </div>
    <p v-else class="text-caption leading-relaxed text-fg-tertiary">{{ t("spaces.knowledge.visual.empty") }}</p>
  </div>
</template>
