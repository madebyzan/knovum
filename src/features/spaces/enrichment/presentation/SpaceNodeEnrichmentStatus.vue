<script setup lang="ts">
import { LoaderCircle, RefreshCw } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../../session/space-store";
import { useNodeEnrichmentStatus } from "../use-node-enrichment-status";

const props = defineProps<{ nodeId: string }>();
const { t } = useI18n();
const { activeSpaceId } = storeToRefs(useSpaceStore());
const nodeId = toRef(props, "nodeId");
const { status, retrying, retryFailed } = useNodeEnrichmentStatus(activeSpaceId, nodeId);
const visible = computed(
  () => status.value.pending || status.value.running || status.value.failed || status.value.waiting,
);
</script>

<template>
  <div
    v-if="visible"
    class="mt-2 flex flex-wrap items-center gap-2 rounded-panel border border-stroke-divider bg-surface-control/25 px-2 py-1.5 text-caption-sm text-fg-secondary"
  >
    <LoaderCircle v-if="status.running" class="size-3 animate-spin text-fg-tertiary" />
    <span v-if="status.pending">{{ t("spaces.processing.queue.pending", { count: status.pending }) }}</span>
    <span v-if="status.waiting">{{ t("spaces.processing.visual.waitingConfiguration") }}</span>
    <span v-if="status.running">{{ t("spaces.processing.queue.running", { count: status.running }) }}</span>
    <span v-if="status.failed" class="text-fg-danger">
      {{ t("spaces.processing.queue.failed", { count: status.failed }) }}
    </span>
    <button
      v-if="status.failed"
      type="button"
      class="ml-auto flex items-center gap-1 rounded-md px-1.5 py-0.5 text-caption-sm text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary"
      :disabled="retrying"
      @click="retryFailed"
    >
      <LoaderCircle v-if="retrying" class="size-3 animate-spin" />
      <RefreshCw v-else class="size-3" />
      {{ t("common.retry") }}
    </button>
    <p v-if="status.failed && status.lastError" class="basis-full text-caption-sm text-fg-danger">
      {{ status.lastError }}
    </p>
  </div>
</template>
