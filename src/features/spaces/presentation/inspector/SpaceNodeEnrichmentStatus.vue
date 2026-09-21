<script setup lang="ts">
import { LoaderCircle, RefreshCw } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, toRef } from "vue";
import { useI18n } from "vue-i18n";

import { useNodeEnrichmentStatus } from "../../enrichment/use-node-enrichment-status";
import { useSpaceStore } from "../../session/space-store";

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
    class="mt-2 flex flex-wrap items-center gap-2 rounded-panel border border-divider bg-control/25 px-2 py-1.5 text-micro text-foreground-secondary"
  >
    <LoaderCircle
      v-if="status.running"
      class="size-(--knovum-icon-size-xs) animate-spin text-foreground-tertiary"
    />
    <span v-if="status.pending">{{
      t("spaces.processing.nodeEnrichmentPending", { count: status.pending })
    }}</span>
    <span v-if="status.waiting">{{ t("spaces.processing.visualWaitingConfiguration") }}</span>
    <span v-if="status.running">{{
      t("spaces.processing.nodeEnrichmentRunning", { count: status.running })
    }}</span>
    <span v-if="status.failed" class="text-danger">
      {{ t("spaces.processing.nodeEnrichmentFailed", { count: status.failed }) }}
    </span>
    <button
      v-if="status.failed"
      type="button"
      class="ml-auto flex items-center gap-1 rounded-control px-1.5 py-0.5 text-micro text-foreground-secondary hover:bg-control-hover hover:text-foreground"
      :disabled="retrying"
      @click="retryFailed"
    >
      <LoaderCircle v-if="retrying" class="size-(--knovum-icon-size-xs) animate-spin" />
      <RefreshCw v-else class="size-(--knovum-icon-size-xs)" />
      {{ t("spaces.processing.nodeEnrichmentRetry") }}
    </button>
    <p v-if="status.failed && status.lastError" class="basis-full text-micro text-danger">
      {{ status.lastError }}
    </p>
  </div>
</template>
