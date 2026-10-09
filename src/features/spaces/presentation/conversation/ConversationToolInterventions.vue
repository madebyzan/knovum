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
  mutationFn: ({ operationId, decision }: { operationId: string; decision: ToolOperationDecision }) =>
    decideConversationToolOperation(operationId, decision),
  onSuccess: () => client.invalidateQueries({ queryKey: ["conversation-tool-interventions"] }),
});
const answering = useMutation({
  mutationFn: ({
    interactionId,
    response,
  }: {
    interactionId: string;
    response: Parameters<typeof answerMcpUserInteraction>[1];
  }) => answerMcpUserInteraction(interactionId, response),
  onSuccess: () => client.invalidateQueries({ queryKey: ["conversation-mcp-interactions"] }),
});
watch(
  () => props.revision,
  () => {
    void client.invalidateQueries({ queryKey: ["conversation-tool-interventions", props.runId] });
    void client.invalidateQueries({ queryKey: ["conversation-mcp-interactions", props.runId] });
  },
);
function decide(operationId: string, decision: ToolOperationDecision) {
  mutation.mutate({ operationId, decision });
}
function answer(interactionId: string, response: Parameters<typeof answerMcpUserInteraction>[1]) {
  answering.mutate({ interactionId, response });
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
            $t(
              operation.state === "prepared"
                ? "spaces.conversation.toolOperations.approval.title"
                : "spaces.conversation.toolOperations.outcomeUnknown.title",
            )
          }}
        </h3>
        <p v-if="operation.state === 'outcome_unknown'" class="text-fg-muted text-caption">
          {{ $t("spaces.conversation.toolOperations.outcomeUnknown.description") }}
        </p>
        <p class="text-caption">
          {{ operation.name }} · {{ $t("spaces.conversation.toolOperations.attempt") }}
          {{ operation.attempt }}
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
            $t("spaces.conversation.toolOperations.approval.allow")
          }}</UiButton
          ><UiButton
            size="sm"
            variant="dangerSubtle"
            :disabled="mutation.isPending.value"
            @click="decide(operation.id, 'deny')"
            >{{ $t("spaces.conversation.toolOperations.approval.deny") }}</UiButton
          >
        </div>
        <div v-else class="flex gap-2">
          <UiButton
            size="sm"
            :disabled="mutation.isPending.value"
            @click="decide(operation.id, 'new_attempt')"
            >{{ $t("spaces.conversation.toolOperations.outcomeUnknown.retry") }}</UiButton
          ><UiButton
            size="sm"
            variant="dangerSubtle"
            :disabled="mutation.isPending.value"
            @click="decide(operation.id, 'stop')"
            >{{ $t("spaces.conversation.toolOperations.outcomeUnknown.stop") }}</UiButton
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
