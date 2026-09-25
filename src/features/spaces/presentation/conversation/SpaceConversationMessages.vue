<script setup lang="ts">
import { AlertCircle, Sparkles } from "@lucide/vue";
import type { ComponentPublicInstance } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { ApprovalSummary } from "../../contracts/generated/ActionApproval";
import type { AssistantTaskUpdate } from "../../contracts/generated/AssistantTaskUpdate";
import type { AssistantMessage } from "../../conversation/conversation-ipc";
import { useConversationScroll } from "../../conversation/use-conversation-scroll";
import SpaceConversationMessage from "./SpaceConversationMessage.vue";
import SpaceConversationPendingActions from "./SpaceConversationPendingActions.vue";

const props = defineProps<{
  messages: readonly AssistantMessage[];
  busy: boolean;
  error?: string | undefined;
  streamedText: string;
  hasOlder: boolean;
  loadingOlder: boolean;
  permission?: Extract<AssistantTaskUpdate, { type: "permission_requested" }> | undefined;
  decidingPermission: boolean;
  approvals: readonly ApprovalSummary[];
  decidingApproval: boolean;
  scrollKey: string;
  savedScrollTop: number;
}>();
const emit = defineEmits<{
  loadOlder: [];
  branch: [message: AssistantMessage];
  regenerate: [message: AssistantMessage];
  decidePermission: [optionId: string | null];
  decideApproval: [approvalId: string, approved: boolean];
  scrollPosition: [key: string, top: number];
}>();
const { scroller, atTail, rows, totalSize, measureElement, updateScrollPosition, scrollToTail, loadOlder } =
  useConversationScroll(props, {
    loadOlder: () => emit("loadOlder"),
    remember: (key, top) => emit("scrollPosition", key, top),
  });

function bindScroller(element: Element | ComponentPublicInstance | null) {
  scroller.value = element instanceof HTMLDivElement ? element : null;
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div v-if="hasOlder" class="shrink-0 p-2 text-center">
      <UiButton size="sm" variant="ghost" :disabled="loadingOlder" @click="loadOlder">
        {{ $t("spaces.assistant.loadOlder") }}
      </UiButton>
    </div>
    <div
      :ref="bindScroller"
      class="min-h-0 flex-1 overflow-y-auto px-4"
      @scroll.passive="updateScrollPosition"
    >
      <div
        v-if="messages.length === 0 && !busy && !error && !permission && !approvals.length"
        class="flex min-h-full flex-col justify-center px-2 pb-16"
      >
        <div class="max-w-72">
          <Sparkles class="mb-4 size-(--knovum-icon-size-md) text-accent" />
          <h2 class="text-title-sm font-semibold text-foreground">{{ $t("spaces.assistant.emptyTitle") }}</h2>
          <p class="mt-2 text-body leading-relaxed text-foreground-secondary">
            {{ $t("spaces.assistant.builtInEmpty") }}
          </p>
          <p class="mt-4 text-caption text-foreground-tertiary">
            {{ $t("spaces.assistant.emptyAttachmentHint") }}
          </p>
        </div>
      </div>
      <div class="relative w-full" :style="{ height: `${totalSize}px` }">
        <article
          v-for="row in rows"
          :key="messages[row.index]!.id"
          :ref="measureElement"
          :data-index="row.index"
          class="absolute top-0 left-0 w-full pb-5"
          :style="{ transform: `translateY(${row.start}px)` }"
        >
          <SpaceConversationMessage
            :message="messages[row.index]!"
            @branch="emit('branch', messages[row.index]!)"
            @regenerate="emit('regenerate', messages[row.index]!)"
          />
        </article>
      </div>
      <article v-if="streamedText" class="pb-5 text-body leading-relaxed whitespace-pre-wrap text-foreground">
        {{ streamedText }}
      </article>
      <SpaceConversationPendingActions
        :permission="permission"
        :deciding-permission="decidingPermission"
        :approvals="approvals"
        :deciding-approval="decidingApproval"
        @decide-permission="emit('decidePermission', $event)"
        @decide-approval="(id, approved) => emit('decideApproval', id, approved)"
      />
      <p
        v-if="busy"
        role="status"
        class="flex items-center gap-2 py-3 text-caption text-foreground-secondary"
      >
        <span class="size-1.5 shrink-0 rounded-full bg-accent" />
        {{ $t("spaces.assistant.agentWorking") }}
      </p>
      <div
        v-if="error"
        role="alert"
        class="mb-4 flex items-start gap-2 rounded-panel border border-danger/30 bg-danger-subtle p-3 text-caption text-danger"
      >
        <AlertCircle class="mt-0.5 size-(--knovum-icon-size-sm) shrink-0" />
        <span class="min-w-0 wrap-break-word">{{ error }}</span>
      </div>
    </div>
    <UiButton
      v-if="!atTail && streamedText"
      size="sm"
      variant="secondary"
      class="mx-auto mb-2"
      @click="scrollToTail"
    >
      {{ $t("spaces.assistant.newMessages") }}
    </UiButton>
  </div>
</template>
