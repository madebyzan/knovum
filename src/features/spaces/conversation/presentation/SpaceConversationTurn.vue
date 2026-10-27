<script setup lang="ts">
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";
import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import type { RunEvent } from "../../contracts/generated/RunEvent";
import { spaceActionLabel } from "../../knowledge-derivations/catalog/action-label";
import { conversationToolItems } from "../conversation-activity-items";
import { conversationTimeline } from "../conversation-timeline";
import { recordedTranscript, type TranscriptStep } from "../conversation-transcript";
import type { ConversationTurn } from "../conversation-turns";
import SpaceConversationActivity from "./SpaceConversationActivity.vue";
import SpaceConversationMessage from "./SpaceConversationMessage.vue";
import SpaceConversationPlan from "./SpaceConversationPlan.vue";
import SpaceConversationShimmerText from "./SpaceConversationShimmerText.vue";

const props = defineProps<{
  turn: ConversationTurn;
  events: readonly RunEvent[];
  messages: readonly ConversationMessage[];
  busy: boolean;
}>();
const emit = defineEmits<{
  branch: [message: ConversationMessage];
  regenerate: [message: ConversationMessage];
  inspectActivity: [runId: string];
}>();
const { t } = useI18n();
const tools = computed(() =>
  props.turn.runId
    ? conversationToolItems({
        events: props.events,
        messages: props.messages,
        runId: props.turn.runId,
        labels: {
          action: (actionId) => spaceActionLabel(actionId),
          default: t("spaces.conversation.activity.tool"),
        },
      })
    : new Map(),
);
const transcript = computed((): readonly TranscriptStep[] => {
  const { live, runId, answers } = props.turn;
  if (live) return live;
  if (!runId) return answers.map((message) => ({ kind: "answer", message }));
  return recordedTranscript(
    { runId, events: props.events, messages: props.messages, answers },
    tools.value.keys(),
  );
});
const blocks = computed(() => conversationTimeline(transcript.value, tools.value));
// Nothing to show yet while the run works. Once there is, the work shows where it happens — a
// tool group says it is running, words arrive as words — so this line never comes back between
// two tools or two messages.
const waiting = computed(() => {
  if (!props.turn.active || blocks.value.length) return undefined;
  return props.turn.runId
    ? "spaces.conversation.activity.generating"
    : "spaces.conversation.conversations.preparing";
});

watch(
  () => props.turn.runId,
  (runId) => {
    if (runId) emit("inspectActivity", runId);
  },
  { immediate: true },
);
</script>

<template>
  <article class="flex min-w-0 flex-col gap-3">
    <SpaceConversationMessage v-if="turn.prompt" :message="turn.prompt" :busy="busy" />
    <div v-if="turn.runId || waiting || blocks.length" class="flex min-w-0 flex-col gap-3">
      <SpaceConversationPlan v-if="turn.runId" :events="events" :active="turn.active" />
      <p v-if="waiting" role="status" class="flex items-center py-1 text-caption text-fg-secondary">
        <SpaceConversationShimmerText indicator>{{ $t(waiting) }}</SpaceConversationShimmerText>
      </p>
      <template v-for="(block, index) in blocks" :key="block.key">
        <SpaceConversationActivity
          v-if="block.kind === 'tools'"
          :items="block.items"
          :active="turn.active"
          :current="turn.active && index === blocks.length - 1"
        />
        <SpaceConversationMessage
          v-else-if="block.kind === 'answer'"
          :message="block.message"
          :actions="block.final"
          :busy="busy"
          @branch="emit('branch', block.message)"
          @regenerate="emit('regenerate', block.message)"
        />
        <UiMarkdown
          v-else
          :content="block.text"
          class="text-body leading-relaxed wrap-break-word text-fg-primary"
        />
      </template>
    </div>
  </article>
</template>
