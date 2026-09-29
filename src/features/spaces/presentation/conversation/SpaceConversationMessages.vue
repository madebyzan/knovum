<script setup lang="ts">
import { AlertCircle } from "@lucide/vue";
import { computed, nextTick, watch, type ComponentPublicInstance } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { AssistantTaskUpdate } from "../../contracts/generated/AssistantTaskUpdate";
import type { PendingApproval } from "../../contracts/generated/PendingApproval";
import type { RunEvent } from "../../contracts/generated/RunEvent";
import type { AssistantMessage } from "../../conversation/conversation-ipc";
import { useConversationScroll } from "../../conversation/use-conversation-scroll";
import SpaceConversationActivity from "./SpaceConversationActivity.vue";
import SpaceConversationEmptyGuide from "./SpaceConversationEmptyGuide.vue";
import SpaceConversationMessage from "./SpaceConversationMessage.vue";
import SpaceConversationPendingActions from "./SpaceConversationPendingActions.vue";
import SpaceConversationShimmerText from "./SpaceConversationShimmerText.vue";

const props = defineProps<{
  messages: readonly AssistantMessage[];
  eventsByRun: Readonly<Record<string, RunEvent[]>>;
  activeRunId?: string | undefined;
  busy: boolean;
  hasDraft: boolean;
  preparing: boolean;
  error?: string | undefined;
  streamedText: string;
  hasOlder: boolean;
  loadingOlder: boolean;
  permission?: Extract<AssistantTaskUpdate, { type: "permission_requested" }> | undefined;
  decidingPermission: boolean;
  approvals: readonly PendingApproval[];
  decidingApproval: boolean;
  scrollKey: string;
  savedScrollTop?: number | undefined;
}>();
const emit = defineEmits<{
  loadOlder: [];
  branch: [message: AssistantMessage];
  regenerate: [message: AssistantMessage];
  decidePermission: [optionId: string | null];
  decideApproval: [approvalId: string, approved: boolean];
  scrollPosition: [key: string, top: number];
  suggest: [prompt: string, selection?: string];
  inspectActivity: [runId: string];
}>();
const visibleMessages = computed(() => props.messages.filter((message) =>
  message.role !== "tool" && (
    message.role !== "assistant" ||
    message.content.parts.some((part) => part.type !== "text" || part.text.trim()) ||
    Boolean(props.streamedText && message.id === props.messages.at(-1)?.id)
  ),
));
const { scroller, atTail, rows, totalSize, measureElement, updateScrollPosition, scrollToTail, loadOlder } =
  useConversationScroll({
    get messages() { return visibleMessages.value; },
    get streamedText() { return props.streamedText; },
    get loadingOlder() { return props.loadingOlder; },
    get scrollKey() { return props.scrollKey; },
    get savedScrollTop() { return props.savedScrollTop; },
  }, {
    loadOlder: () => emit("loadOlder"),
    remember: (key, top) => emit("scrollPosition", key, top),
  });
const streamingMessageId = computed(() => {
  const last = visibleMessages.value.at(-1);
  return last?.role === "assistant" ? last.id : undefined;
});
const hasActiveTurn = computed(() => Boolean(props.activeRunId && props.messages.some((message) =>
  message.role === "user" && message.content.runId === props.activeRunId,
)));
watch(() => props.eventsByRun, async () => {
  if (!atTail.value) return;
  await nextTick();
  scrollToTail();
});

function bindScroller(element: Element | ComponentPublicInstance | null) {
  scroller.value = element instanceof HTMLDivElement ? element : null;
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div v-if="hasOlder" class="shrink-0 p-2 text-center">
      <UiButton size="sm" variant="ghost" :disabled="loadingOlder" @click="loadOlder">
        {{ $t("spaces.assistant.messages.loadOlder") }}
      </UiButton>
    </div>
    <div
      :ref="bindScroller"
      class="min-h-0 flex-1 overflow-y-auto px-4 pt-5 pb-4"
      @scroll.passive="updateScrollPosition"
    >
      <div
        v-if="visibleMessages.length === 0 && !hasDraft && !busy && !error && !permission && !approvals.length"
        class="flex min-h-full items-center justify-center px-2 pb-12"
      >
        <SpaceConversationEmptyGuide @select="(value, selection) => emit('suggest', value, selection)" />
      </div>
      <div class="relative w-full" :style="{ height: `${totalSize}px` }">
        <article
          v-for="row in rows"
          :key="visibleMessages[row.index]!.id"
          :ref="measureElement"
          :data-index="row.index"
          class="absolute top-0 left-0 w-full pb-5"
          :style="{ transform: `translateY(${row.start}px)` }"
        >
          <SpaceConversationMessage
            :message="visibleMessages[row.index]!"
            :streamed-text="visibleMessages[row.index]!.id === streamingMessageId ? streamedText : ''"
            :busy="busy"
            @branch="emit('branch', visibleMessages[row.index]!)"
            @regenerate="emit('regenerate', visibleMessages[row.index]!)"
          />
          <SpaceConversationActivity
            v-if="visibleMessages[row.index]!.role === 'user' && visibleMessages[row.index]!.content.runId"
            :run-id="visibleMessages[row.index]!.content.runId!"
            :events="eventsByRun[visibleMessages[row.index]!.content.runId!] ?? []"
            :messages="messages"
            :active="busy && activeRunId === visibleMessages[row.index]!.content.runId"
            @inspect="emit('inspectActivity', $event)"
          />
        </article>
      </div>
      <article v-if="streamedText && !streamingMessageId" class="pb-5 text-body leading-relaxed text-fg-primary">
        <p class="whitespace-pre-wrap">{{ streamedText }}</p>
      </article>
      <SpaceConversationPendingActions
        :permission="permission"
        :deciding-permission="decidingPermission"
        :approvals="approvals"
        :deciding-approval="decidingApproval"
        @decide-permission="emit('decidePermission', $event)"
        @decide-approval="(id, approved) => emit('decideApproval', id, approved)"
      />
      <p v-if="busy && !hasActiveTurn && !error" role="status" class="py-3 text-caption text-fg-secondary">
        <SpaceConversationShimmerText>
          {{ $t(preparing ? "spaces.assistant.conversations.preparing" : "spaces.assistant.activity.generating") }}
        </SpaceConversationShimmerText>
      </p>
      <div
        v-if="error"
        role="alert"
        class="mb-4 flex items-start gap-2 rounded-panel border border-stroke-danger/30 bg-surface-danger-subtle p-3 text-caption text-fg-danger"
      >
        <AlertCircle class="mt-0.5 size-3.5 shrink-0" />
        <span class="min-w-0 wrap-break-word">{{ error }}</span>
      </div>
    </div>
    <UiButton
      v-if="!atTail && (visibleMessages.length || streamedText)"
      size="sm"
      variant="secondary"
      class="mx-auto mb-2"
      @click="scrollToTail"
    >
      {{ $t("spaces.assistant.messages.new") }}
    </UiButton>
  </div>
</template>
