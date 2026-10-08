<script setup lang="ts">
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, ref, watch } from "vue";
import { z } from "zod";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { RunEvent } from "../../contracts/generated/RunEvent";
import { listConversationToolOperations } from "../../conversation/tool-operation-ipc";

const props = defineProps<{ runId: string; events: readonly RunEvent[]; active: boolean }>();
const open = ref(false);
const client = useQueryClient();
const query = useQuery({
  queryKey: computed(() => ["conversation-tool-interventions", props.runId]),
  queryFn: () => listConversationToolOperations(props.runId),
  enabled: open,
});
watch(
  () => props.events.length,
  () => {
    if (open.value)
      void client.invalidateQueries({ queryKey: ["conversation-tool-interventions", props.runId] });
  },
);
const progressSchema = z.object({
  progressToken: z.string(),
  progress: z.number(),
  total: z.number().optional(),
  message: z.string().optional(),
});
const progressByOperation = computed(() => {
  const progress = new Map<string, z.infer<typeof progressSchema>>();
  for (const event of props.events) {
    if (event.kind !== "mcp_progress") continue;
    const parsed = progressSchema.safeParse(event.payload);
    if (parsed.success) progress.set(parsed.data.progressToken, parsed.data);
  }
  return progress;
});
const latestProgress = computed(() => [...progressByOperation.value.values()].at(-1));
</script>

<template>
  <p v-if="active && latestProgress" role="status" class="mt-2 text-caption text-fg-secondary">
    {{ $t("extensions.progress") }}: {{ latestProgress.progress
    }}<template v-if="latestProgress.total !== undefined"> / {{ latestProgress.total }}</template>
    {{ latestProgress.message }}
  </p>
  <details
    class="mt-2 text-caption text-fg-secondary"
    data-testid="conversation-tool-history"
    @toggle="open = ($event.target as HTMLDetailsElement).open"
  >
    <summary class="cursor-pointer py-1">{{ $t("extensions.operationHistory") }}</summary>
    <p v-if="query.isPending.value">{{ $t("extensions.loading") }}</p>
    <p v-else-if="query.error.value" role="alert" class="text-fg-danger">
      {{ getErrorMessage(query.error.value) }}
    </p>
    <p v-else-if="!query.data.value?.length">{{ $t("extensions.noOperations") }}</p>
    <article
      v-for="operation in query.data.value"
      :key="operation.id"
      class="space-y-1 border-l border-stroke-divider py-2 pl-3"
    >
      <p>
        {{ operation.name }} · {{ $t("extensions.attempt") }} {{ operation.attempt }} · {{ operation.state }}
      </p>
      <p v-if="progressByOperation.has(operation.id)">
        {{ $t("extensions.progress") }}: {{ progressByOperation.get(operation.id)?.progress
        }}<template v-if="progressByOperation.get(operation.id)?.total !== undefined">
          / {{ progressByOperation.get(operation.id)?.total }}</template
        >
        {{ progressByOperation.get(operation.id)?.message }}
      </p>
      <pre class="max-h-64 overflow-auto break-all whitespace-pre-wrap">{{
        JSON.stringify(
          {
            source: operation.source,
            context: operation.context,
            approval: operation.approval,
            resolution: operation.resolution,
            output: operation.output,
          },
          null,
          2,
        )
      }}</pre>
    </article>
  </details>
</template>
