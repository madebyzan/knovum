<script setup lang="ts">
import { storeToRefs } from "pinia";
import { nextTick, useTemplateRef } from "vue";

import { useConversationSessionStore } from "../../conversation/conversation-session-store";
import SpaceConversationComposer from "./SpaceConversationComposer.vue";
import SpaceConversationHeader from "./SpaceConversationHeader.vue";
import SpaceConversationMessages from "./SpaceConversationMessages.vue";
import SpaceConversationRunStatus from "./SpaceConversationRunStatus.vue";

const conversationSession = useConversationSessionStore();
const {
  messages,
  selectedConversationId,
  prompt,
  target,
  activeRunId,
  activeConversationId,
  pendingPermission,
  approvals,
  decidingApproval,
  visibleTaskState,
  visibleWaitReason,
  loading,
  loadingOlder,
  hasOlder,
  visibleError,
  visiblePermission,
  decidingPermission,
  visibleStreamedText,
  visibleAttachments,
  attachmentCandidates,
  attachmentError,
  pickingFiles,
  scrollPositions,
} = storeToRefs(conversationSession);
const composer = useTemplateRef<InstanceType<typeof SpaceConversationComposer>>("composer");

async function useSuggestion(value: string) {
  prompt.value = value;
  await nextTick();
  composer.value?.focus();
}
</script>

<template>
  <section class="flex h-full min-h-0 min-w-0 flex-col bg-surface-panel">
    <SpaceConversationHeader />
    <button
      v-if="(pendingPermission || approvals.length) && activeConversationId !== selectedConversationId"
      type="button"
      class="shrink-0 border-b border-stroke-warning/40 bg-surface-warning-subtle px-3 py-2 text-left text-caption-md text-fg-primary hover:bg-surface-warning-subtle-hover"
      @click="conversationSession.selectConversation(activeConversationId)"
    >
      {{ $t("spaces.assistant.permission.return") }}
    </button>
    <button
      v-else-if="activeRunId && activeConversationId && activeConversationId !== selectedConversationId"
      type="button"
      class="shrink-0 border-b border-stroke-divider bg-surface-control px-3 py-2 text-left text-caption-md text-fg-primary hover:bg-surface-control-hover"
      @click="conversationSession.selectConversation(activeConversationId)"
    >
      {{ $t("spaces.assistant.conversations.returnToRunning") }}
    </button>
    <div class="min-h-0 min-w-0 flex-1">
      <SpaceConversationMessages
        class="h-full"
        :scroll-key="selectedConversationId"
        :saved-scroll-top="scrollPositions[selectedConversationId] ?? 0"
        :messages="messages"
        :busy="loading"
        :preparing="loading && !activeRunId"
        :error="visibleError"
        :streamed-text="visibleStreamedText"
        :has-older="hasOlder"
        :loading-older="loadingOlder"
        :permission="visiblePermission"
        :deciding-permission="decidingPermission"
        :approvals="activeConversationId === selectedConversationId ? approvals : []"
        :deciding-approval="decidingApproval"
        @load-older="conversationSession.loadOlder"
        @branch="conversationSession.branchFrom"
        @regenerate="conversationSession.regenerate"
        @decide-permission="conversationSession.decidePermission"
        @decide-approval="conversationSession.decideApproval"
        @scroll-position="conversationSession.rememberScroll"
        @suggest="useSuggestion"
      />
    </div>
    <SpaceConversationRunStatus
      :state="visibleTaskState"
      :wait-reason="visibleWaitReason"
      :has-pending-decision="Boolean(visiblePermission || approvals.length)"
      @resume="conversationSession.resume"
    />
    <SpaceConversationComposer
      ref="composer"
      v-model="prompt"
      v-model:target="target"
      :attachments="visibleAttachments"
      :attachment-candidates="attachmentCandidates"
      :attachment-error="attachmentError"
      :picking-files="pickingFiles"
      :busy="loading || Boolean(activeRunId)"
      @add-selection="conversationSession.addSelectedAttachments"
      @add-attachment="(id) => conversationSession.addNodeAttachments([id])"
      @choose-files="conversationSession.chooseFiles"
      @remove-attachment="conversationSession.removeAttachment"
      @send="conversationSession.send"
      @cancel="conversationSession.cancel"
    />
  </section>
</template>
