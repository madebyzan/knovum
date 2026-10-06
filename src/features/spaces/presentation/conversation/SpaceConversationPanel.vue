<script setup lang="ts">
import { storeToRefs } from "pinia";
import { nextTick, useTemplateRef } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useConversationSessionStore } from "../../conversation/conversation-session-store";
import SpaceConversationComposer from "./SpaceConversationComposer.vue";
import SpaceConversationHeader from "./SpaceConversationHeader.vue";
import SpaceConversationMessages from "./SpaceConversationMessages.vue";
import SpaceConversationRunStatus from "./SpaceConversationRunStatus.vue";

const conversationSession = useConversationSessionStore();
const {
  messages,
  eventsByRun,
  activityError,
  selectedConversationId,
  mode,
  prompt,
  target,
  activeRunId,
  activeConversationId,
  visibleTaskState,
  visibleWaitReason,
  canPause,
  loading,
  loadingOlder,
  hasOlder,
  visibleError,
  visibleStreamedText,
  visibleAttachments,
  attachmentCandidates,
  attachmentError,
  pickingFiles,
  scrollPositions,
} = storeToRefs(conversationSession);
const composer = useTemplateRef<InstanceType<typeof SpaceConversationComposer>>("composer");

async function applySuggestion(value: string, selection?: string) {
  prompt.value = value;
  await nextTick();
  composer.value?.focus(selection);
}

async function previewAttachment(key: string) {
  try {
    await conversationSession.previewAttachment(key);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}
</script>

<template>
  <section class="flex h-full min-h-0 min-w-0 flex-col bg-surface-panel">
    <SpaceConversationHeader />
    <button
      v-if="activeRunId && activeConversationId && activeConversationId !== selectedConversationId"
      type="button"
      class="shrink-0 border-b border-stroke-divider bg-surface-control px-3 py-2 text-left text-caption text-fg-primary hover:bg-surface-control-hover"
      @click="conversationSession.selectConversation(activeConversationId)"
    >
      {{ $t("spaces.assistant.conversations.returnToRunning") }}
    </button>
    <div class="min-h-0 min-w-0 flex-1">
      <SpaceConversationMessages
        class="h-full"
        :scroll-key="selectedConversationId"
        :saved-scroll-top="scrollPositions[selectedConversationId]"
        :messages="messages"
        :events-by-run="eventsByRun"
        :active-run-id="activeRunId"
        :busy="loading"
        :has-draft="Boolean(prompt.trim())"
        :preparing="loading && !activeRunId"
        :error="visibleError || activityError"
        :streamed-text="visibleStreamedText"
        :has-older="hasOlder"
        :loading-older="loadingOlder"
        @load-older="conversationSession.loadOlder"
        @branch="conversationSession.branchFrom"
        @regenerate="conversationSession.regenerate"
        @scroll-position="conversationSession.rememberScroll"
        @suggest="applySuggestion"
        @inspect-activity="conversationSession.inspectActivity"
      />
    </div>
    <SpaceConversationRunStatus
      :can-pause="canPause"
      @pause="conversationSession.pause"
      :state="visibleTaskState"
      :wait-reason="visibleWaitReason"
      @resume="conversationSession.resume"
      @retry-model="conversationSession.retryModel"
    />
    <SpaceConversationComposer
      ref="composer"
      v-model="prompt"
      v-model:mode="mode"
      v-model:target="target"
      :attachments="visibleAttachments"
      :attachment-candidates="attachmentCandidates"
      :attachment-error="attachmentError"
      :picking-files="pickingFiles"
      :busy="loading || Boolean(activeRunId)"
      @add-selection="conversationSession.addSelectedAttachments"
      @add-attachment="(id) => conversationSession.addNodeAttachments([id])"
      @choose-files="conversationSession.chooseFiles"
      @drop-files="conversationSession.chooseFiles($event)"
      @remove-attachment="conversationSession.removeAttachment"
      @preview-attachment="previewAttachment"
      @send="conversationSession.send"
      @cancel="conversationSession.cancel"
    />
  </section>
</template>
