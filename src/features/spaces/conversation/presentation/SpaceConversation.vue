<script setup lang="ts">
import { CornerUpLeft } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, nextTick, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useConversationSessionStore } from "../conversation-session-store";
import AgentExtensionPicker from "./AgentExtensionPicker.vue";
import ConversationToolInterventions from "./ConversationToolInterventions.vue";
import SpaceConversationComposer from "./SpaceConversationComposer.vue";
import SpaceConversationHeader from "./SpaceConversationHeader.vue";
import SpaceConversationMessages from "./SpaceConversationMessages.vue";
import SpaceConversationRunStatus from "./SpaceConversationRunStatus.vue";

const conversationSession = useConversationSessionStore();
const {
  messages,
  extensionSelection,
  extensionAvailability,
  extensionsLoading,
  extensionsError,
  eventsByRunId,
  activityError,
  selectedConversationId,
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
  visibleLive,
  visibleAttachments,
  attachmentCandidates,
  attachmentError,
  draftError,
  restorationError,
  restoringDraft,
  reacquireFileNames,
  pickingFiles,
  scrollTopByConversationId,
} = storeToRefs(conversationSession);
const composer = useTemplateRef<InstanceType<typeof SpaceConversationComposer>>("composer");
const { t } = useI18n();
// Three different things were one "busy" flag: a conversation that cannot take a question yet, an
// answer being produced, and a run that exists and can therefore be stopped. The composer needs
// all three kept apart — otherwise loading an extension list offers a stop button that stops
// nothing.
const composerDisabled = computed(
  () =>
    restoringDraft.value ||
    Boolean(restorationError.value) ||
    extensionsLoading.value ||
    Boolean(extensionsError.value),
);
const running = computed(() => loading.value || Boolean(activeRunId.value));
const producing = computed(
  () =>
    running.value &&
    (!visibleTaskState.value ||
      visibleTaskState.value === "queued" ||
      visibleTaskState.value === "running" ||
      (visibleTaskState.value === "waiting" && visibleWaitReason.value === "executor")),
);
const composerError = computed(() => {
  if (restorationError.value) return restorationError.value;
  if (draftError.value) return draftError.value;
  if (reacquireFileNames.value.length)
    return t("spaces.conversation.uploads.reacquire", { names: reacquireFileNames.value.join(", ") });
  return attachmentError.value;
});

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
      class="flex shrink-0 items-center gap-1.5 border-b border-stroke-divider bg-surface-control px-3 py-2 text-left text-caption text-fg-primary hover:bg-surface-control-hover"
      @click="conversationSession.selectConversation(activeConversationId)"
    >
      <CornerUpLeft class="size-3.5 shrink-0 text-fg-tertiary" />
      {{ $t("spaces.conversation.conversations.returnToRunning") }}
    </button>
    <div class="min-h-0 min-w-0 flex-1">
      <SpaceConversationMessages
        class="h-full"
        :scroll-key="selectedConversationId"
        :saved-scroll-top="scrollTopByConversationId[selectedConversationId]"
        :messages="messages"
        :events-by-run-id="eventsByRunId"
        :active-run-id="activeRunId"
        :busy="loading"
        :producing="producing"
        :has-draft="Boolean(prompt.trim())"
        :preparing="loading && !activeRunId"
        :live="visibleLive"
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
    <ConversationToolInterventions
      v-if="activeRunId && visibleTaskState === 'waiting'"
      :run-id="activeRunId"
      :revision="conversationSession.activityRevision"
    />
    <SpaceConversationRunStatus
      :state="visibleTaskState"
      :wait-reason="visibleWaitReason"
      :error="visibleError || activityError"
      @resume="conversationSession.resume"
      @retry-model="conversationSession.retryModel"
    />
    <SpaceConversationComposer
      ref="composer"
      v-model="prompt"
      v-model:target="target"
      :attachments="visibleAttachments"
      :attachment-candidates="attachmentCandidates"
      :attachment-error="composerError"
      :picking-files="pickingFiles"
      :unresolved-file-names="reacquireFileNames"
      :running="running"
      :cancellable="Boolean(activeRunId)"
      :disabled="composerDisabled"
      :can-pause="canPause"
      @add-selection="conversationSession.addSelectedAttachments"
      @add-attachment="(id) => conversationSession.addNodeAttachments([id])"
      @choose-files="conversationSession.chooseFiles"
      @drop-files="conversationSession.chooseFiles($event)"
      @discard-file-reference="conversationSession.discardFileReference"
      @remove-attachment="conversationSession.removeAttachment"
      @preview-attachment="previewAttachment"
      @send="conversationSession.send"
      @cancel="conversationSession.cancel"
      @pause="conversationSession.pause"
    >
      <template #extensions>
        <AgentExtensionPicker
          v-model="extensionSelection"
          :availability="extensionAvailability"
          :disabled="composerDisabled || running"
          :error="extensionsError"
        />
      </template>
    </SpaceConversationComposer>
  </section>
</template>
