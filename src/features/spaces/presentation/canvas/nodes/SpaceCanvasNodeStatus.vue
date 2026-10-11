<script setup lang="ts">
import { CircleAlert, Clock3 } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, inject } from "vue";
import { useI18n } from "vue-i18n";

import { enrichmentProjectionKey } from "../../../enrichment/enrichment-projection-key";
import { enrichmentJobTitle } from "../../../enrichment/job-tasks";
import { useImageEditSessionStore } from "../../../image-editing/image-edit-session-store";
import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import { useSpaceStore } from "../../../session/space-store";
import { canvasNodeWork } from "./canvas-node-work";
import SpaceCanvasEditorIndicator from "./SpaceCanvasEditorIndicator.vue";
import SpaceCanvasWorkEffect from "./SpaceCanvasWorkEffect.vue";

const props = defineProps<{ id: string; data: SpaceNodePayload }>();
const { t } = useI18n();
const { activeSpaceId, mediaAnalysisIds } = storeToRefs(useSpaceStore());
const enrichmentJobs = inject(enrichmentProjectionKey);
if (!enrichmentJobs) throw new Error("Canvas enrichment projection is unavailable");
const imageEditSession = useImageEditSessionStore();
const enrichment = computed(() => {
  const jobs = enrichmentJobs.value.filter(
    ({ job }) => job.spaceId === activeSpaceId.value && job.nodeId === props.id,
  );
  const rank = { failed: 0, awaiting_configuration: 1, running: 2, pending: 3, succeeded: 4, cancelled: 5 };
  return jobs.toSorted((a, b) => rank[a.job.status] - rank[b.job.status])[0]?.job;
});
const enrichmentLabel = computed(() =>
  enrichment.value
    ? `${enrichmentJobTitle(enrichment.value.jobType)} · ${t(`spaces.processing.jobStatus.${enrichment.value.status}`)}`
    : "",
);
const work = computed(() =>
  canvasNodeWork(props.data, {
    mediaAnalysis: mediaAnalysisIds.value.includes(props.id) && !enrichment.value,
    imageEditApplying: imageEditSession.applying && imageEditSession.active?.nodeId === props.id,
  }),
);
const label = computed(() => (work.value ? t(work.value.labelKey) : ""));
</script>

<template>
  <SpaceCanvasEditorIndicator :node-id="id" />
  <span v-if="enrichment" :title="enrichmentLabel" :aria-label="enrichmentLabel" role="status">
    <CircleAlert v-if="enrichment.status === 'failed'" class="size-3 text-fg-danger" :stroke-width="1.6" />
    <CircleAlert
      v-else-if="enrichment.status === 'awaiting_configuration'"
      class="size-3 text-fg-warning"
      :stroke-width="1.6"
    />
    <Clock3 v-else-if="enrichment.status === 'pending'" class="size-3 text-fg-tertiary" :stroke-width="1.6" />
    <SpaceCanvasWorkEffect v-else placement="inline" />
  </span>
  <span v-if="work?.surface === 'chrome'" :title="label" :aria-label="label" role="status">
    <CircleAlert v-if="work.phase === 'failed'" class="size-3 text-fg-danger" :stroke-width="1.6" />
    <SpaceCanvasWorkEffect v-else placement="inline" />
  </span>
  <span v-else-if="work?.surface === 'selection'" class="text-caption-sm text-fg-accent" role="status">
    {{ label }}
  </span>
</template>
