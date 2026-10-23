<script setup lang="ts">
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";
import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import type { RunEvent } from "../../contracts/generated/RunEvent";
import { spaceActionLabel } from "../../knowledge-derivations/catalog/action-label";
import { conversationActivityItems } from "../conversation-activity-items";
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
// The activity is read once here and handed to the list that shows it, so the turn can tell
// whether anything has happened yet without asking the same question twice.
const items = computed(() =>
  props.turn.runId
    ? conversationActivityItems({
        events: props.events,
        messages: props.messages,
        runId: props.turn.runId,
        labels: {
          action: (actionId) => spaceActionLabel(actionId),
          default: t("spaces.conversation.activity.tool"),
        },
      })
    : [],
);

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
    <div v-if="turn.runId || turn.pending || turn.answers.length" class="flex min-w-0 flex-col gap-3">
      <SpaceConversationPlan v-if="turn.runId" :events="events" />
      <SpaceConversationActivity v-if="items.length" :items="items" :active="turn.active" />
      <p v-else-if="turn.pending" role="status" class="text-caption text-fg-secondary">
        <SpaceConversationShimmerText>
          {{
            $t(
              turn.pending === "preparing"
                ? "spaces.conversation.conversations.preparing"
                : "spaces.conversation.activity.generating",
            )
          }}
        </SpaceConversationShimmerText>
      </p>
      <SpaceConversationMessage
        v-for="(answer, index) in turn.answers"
        :key="answer.id"
        :message="answer"
        :streamed-text="index === turn.answers.length - 1 ? turn.streamedText : ''"
        :busy="busy"
        @branch="emit('branch', answer)"
        @regenerate="emit('regenerate', answer)"
      />
      <!-- The first words of an answer arrive before the message that will hold them exists. -->
      <UiMarkdown
        v-if="turn.streamedText && !turn.answers.length"
        :content="turn.streamedText"
        class="text-body leading-relaxed wrap-break-word text-fg-primary"
      />
    </div>
  </article>
</template>
