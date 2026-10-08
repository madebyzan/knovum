<script setup lang="ts">
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, watch } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { ToolOperationDecision } from "../../contracts/generated/ToolOperationDecision";
import {
  listConversationToolOperations,
  decideConversationToolOperation,
  listMcpUserInteractions,
  answerMcpUserInteraction,
} from "../../conversation/tool-operation-ipc";
import McpElicitationForm from "./McpElicitationForm.vue";
const props = defineProps<{ runId: string; revision: number }>();
const client = useQueryClient();
const query = useQuery({
  queryKey: computed(() => ["conversation-tool-interventions", props.runId]),
  queryFn: () => listConversationToolOperations(props.runId),
});
const interactions = useQuery({
  queryKey: computed(() => ["conversation-mcp-interactions", props.runId]),
  queryFn: () => listMcpUserInteractions(props.runId),
});
const mutation = useMutation({
  mutationFn: ({ id, decision }: { id: string; decision: ToolOperationDecision }) =>
    decideConversationToolOperation(id, decision),
  onSuccess: () => client.invalidateQueries({ queryKey: ["conversation-tool-interventions"] }),
});
const answering = useMutation({
  mutationFn: ({ id, response }: { id: string; response: Parameters<typeof answerMcpUserInteraction>[1] }) =>
    answerMcpUserInteraction(id, response),
  onSuccess: () => client.invalidateQueries({ queryKey: ["conversation-mcp-interactions"] }),
});
watch(
  () => props.revision,
  () => {
    void client.invalidateQueries({ queryKey: ["conversation-tool-interventions", props.runId] });
    void client.invalidateQueries({ queryKey: ["conversation-mcp-interactions", props.runId] });
  },
);
function decide(id: string, decision: ToolOperationDecision) {
  mutation.mutate({ id, decision });
}
function answer(id: string, response: Parameters<typeof answerMcpUserInteraction>[1]) {
  answering.mutate({ id, response });
}
</script>

<template>
  <section class="max-h-64 shrink-0 overflow-auto" data-testid="conversation-tool-interventions">
    <p
      v-if="query.error.value || interactions.error.value || mutation.error.value || answering.error.value"
      role="alert"
      class="px-3 text-caption text-fg-danger"
    >
      {{
        getErrorMessage(
          query.error.value || interactions.error.value || mutation.error.value || answering.error.value,
        )
      }}
    </p>
    <template v-for="operation in query.data.value" :key="operation.id">
      <article
        v-if="
          (operation.state === 'prepared' && !operation.approval) ||
          (operation.state === 'outcome_unknown' && !operation.resolution)
        "
        class="space-y-2 border-t border-stroke-divider p-3"
      >
        <h3 class="text-body font-medium">
          {{
            $t(operation.state === "prepared" ? "extensions.toolApproval" : "extensions.toolOutcomeUnknown")
          }}
        </h3>
        <p class="text-caption">
          {{ operation.name }} · {{ $t("extensions.attempt") }} {{ operation.attempt }}
        </p>
        <pre class="max-h-32 overflow-auto rounded-sm bg-surface-control p-2 text-caption">{{
          JSON.stringify(
            { source: operation.source, context: operation.context, output: operation.output },
            null,
            2,
          )
        }}</pre>
        <div v-if="operation.state === 'prepared'" class="flex gap-2">
          <UiButton size="sm" :disabled="mutation.isPending.value" @click="decide(operation.id, 'approve')">{{
            $t("extensions.approve")
          }}</UiButton
          ><UiButton
            size="sm"
            variant="dangerSubtle"
            :disabled="mutation.isPending.value"
            @click="decide(operation.id, 'deny')"
            >{{ $t("extensions.denyOperation") }}</UiButton
          >
        </div>
        <div v-else class="flex gap-2">
          <UiButton
            size="sm"
            :disabled="mutation.isPending.value"
            @click="decide(operation.id, 'new_attempt')"
            >{{ $t("extensions.newAttempt") }}</UiButton
          ><UiButton
            size="sm"
            variant="dangerSubtle"
            :disabled="mutation.isPending.value"
            @click="decide(operation.id, 'stop')"
            >{{ $t("extensions.stop") }}</UiButton
          >
        </div>
      </article>
    </template>
    <McpElicitationForm
      v-for="interaction in interactions.data.value"
      :key="interaction.id"
      :interaction="interaction"
      :busy="answering.isPending.value"
      @answer="answer"
    />
  </section>
</template>
