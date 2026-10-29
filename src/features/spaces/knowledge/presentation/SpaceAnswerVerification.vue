<script setup lang="ts">
import { ChevronRight, RefreshCw, ShieldCheck } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { StatementEvidence } from "../../contracts/generated/StatementEvidence";
import type { StatementSupport } from "../../contracts/generated/StatementSupport";
import { useSpaceStore } from "../../session/space-store";
import { useKnowledgeProposals } from "../proposals/use-knowledge-proposals";
import { isGap, isRecordable } from "../verification/answer-verification-view";
import { useAnswerVerification } from "../verification/use-answer-verification";
import SpaceAnswerStatement from "./SpaceAnswerStatement.vue";
import SpaceKnowledgeProposalReview from "./SpaceKnowledgeProposalReview.vue";

/**
 * Which sentences of an answer its sources state. Gaps are listed first; supported statements can
 * be recorded as Claims, which reach the Knowledge graph only once the person accepts them.
 */
const props = defineProps<{ spaceId: string; conversationId: string; messageId: string }>();
const emit = defineEmits<{ open: [evidence: StatementEvidence] }>();
const { t } = useI18n();
const store = useSpaceStore();
const SKIPS = {
  "no-sources": "spaces.knowledge.verification.skip.noSources",
  "no-statements": "spaces.knowledge.verification.skip.noStatements",
  "no-target": "spaces.knowledge.verification.skip.noTarget",
} as const;
const STATES = {
  pending: "spaces.knowledge.verification.state.pending",
  running: "spaces.knowledge.verification.state.running",
  failed: "spaces.knowledge.verification.state.failed",
} as const;
const { verification, views, counts, verify, correct, record } = useAnswerVerification(
  () => props.spaceId,
  () => props.messageId,
);
const { proposals, accept, dismiss } = useKnowledgeProposals(
  () => props.spaceId,
  () => ({ kind: "answer", conversationId: props.conversationId, messageId: props.messageId }),
);
const chosen = shallowRef(new Set<number>());
const busy = shallowRef(false);
const ordered = computed(() => views.value.toSorted((a, b) => Number(isGap(b)) - Number(isGap(a))));
const state = computed(() => verification.value?.state ?? null);
const summary = computed(() => {
  const current = verification.value;
  if (!current) return null;
  if (current.state === "completed")
    return t("spaces.knowledge.verification.summary", {
      supported: counts.value.supported,
      checked: counts.value.checked,
    });
  if (current.state === "skipped") return t(SKIPS[current.skip ?? "no-sources"]);
  return t(STATES[current.state]);
});

async function run<T>(work: () => Promise<T>) {
  busy.value = true;
  try {
    return await work();
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    busy.value = false;
  }
}

function choose(statement: number, value: boolean) {
  const next = new Set(chosen.value);
  if (value) next.add(statement);
  else next.delete(statement);
  chosen.value = next;
}

const correctSupport = (statement: number, support: StatementSupport) =>
  run(() => correct(statement, support));
// Accepted Claims land on the canvas, so the open Space is read again.
const acceptProposal = (proposalId: string, changes: number[] | null) =>
  run(async () => {
    await accept(proposalId, changes);
    await store.refreshActiveSpace();
  });
const recordChosen = () =>
  run(async () => {
    await record([...chosen.value]);
    chosen.value = new Set();
  });
</script>

<template>
  <div v-if="verification" class="w-full space-y-2 text-caption">
    <details v-if="state === 'completed' && views.length" class="group/verification w-full">
      <summary
        class="inline-flex cursor-pointer list-none items-center gap-1 rounded-md px-1 py-0.5 text-fg-secondary select-none hover:text-fg-primary [&::-webkit-details-marker]:hidden"
      >
        <ShieldCheck class="size-3.5" />
        {{ summary }}
        <ChevronRight class="size-3.5 transition-transform group-open/verification:rotate-90" />
      </summary>
      <p v-if="verification.truncated" class="px-1 pt-1 text-fg-tertiary">
        {{ t("spaces.knowledge.verification.truncated") }}
      </p>
      <ol class="mt-1 divide-y divide-stroke-divider px-1">
        <SpaceAnswerStatement
          v-for="view in ordered"
          :key="view.statement.id"
          :view="view"
          :recordable="isRecordable(view)"
          :chosen="chosen.has(view.statement.id)"
          @update:chosen="choose(view.statement.id, $event)"
          @correct="correctSupport(view.statement.id, $event)"
          @open="emit('open', $event)"
        />
      </ol>
      <div class="flex justify-end pt-1">
        <UiButton size="sm" :disabled="busy || !chosen.size" @click="recordChosen">
          {{ t("spaces.knowledge.verification.record", { count: chosen.size }) }}
        </UiButton>
      </div>
    </details>
    <div v-else class="flex items-center gap-1 px-1 text-fg-tertiary">
      <ShieldCheck class="size-3.5" />
      <span :class="{ 'text-fg-danger': state === 'failed' }">{{ summary }}</span>
      <UiIconButton
        v-if="state === 'failed' || state === 'skipped'"
        :label="t('spaces.knowledge.verification.retry')"
        tooltip
        size="sm"
        :disabled="busy"
        @click="run(verify)"
      >
        <RefreshCw class="size-3.5" />
      </UiIconButton>
    </div>
    <SpaceKnowledgeProposalReview
      v-for="proposal in proposals ?? []"
      :key="proposal.id"
      :proposal="proposal"
      :busy="busy"
      @accept="acceptProposal(proposal.id, $event)"
      @dismiss="run(() => dismiss(proposal.id))"
    />
  </div>
</template>
