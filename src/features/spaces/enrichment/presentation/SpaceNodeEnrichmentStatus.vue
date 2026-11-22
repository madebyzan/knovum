<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, toRef } from "vue";

import { useSpaceStore } from "../../session/space-store";
import { MEDIA_ENRICHMENT_JOB_TYPES, useNodeEnrichmentStatus } from "../use-node-enrichment-status";
import SpaceEnrichmentJobStatus from "./SpaceEnrichmentJobStatus.vue";

const ACTIVE_STATUSES = new Set(["pending", "running", "awaiting_configuration", "failed"]);

const props = defineProps<{ nodeId: string }>();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const nodeId = toRef(props, "nodeId");
const { jobs, refresh } = useNodeEnrichmentStatus(activeSpaceId, nodeId, MEDIA_ENRICHMENT_JOB_TYPES);
const visibleJobs = computed(() => jobs.value.filter((job) => ACTIVE_STATUSES.has(job.status)));
</script>

<template>
  <div v-if="visibleJobs.length" class="space-y-1.5">
    <SpaceEnrichmentJobStatus
      v-for="job in visibleJobs"
      :key="job.id"
      :job="job"
      class="mt-0!"
      @retried="refresh"
    />
  </div>
</template>
