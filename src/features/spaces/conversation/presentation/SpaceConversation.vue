<script setup lang="ts">
import { CornerUpLeft } from "@lucide/vue";
import { computed, nextTick, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import { quotedFollowUp } from "../answer-capture";
import { useConversationSessionStore } from "../conversation-session-store";
import AgentExtensionPicker from "./AgentExtensionPicker.vue";
import ConversationToolInterventions from "./ConversationToolInterventions.vue";
import SpaceConversationComposer from "./SpaceConversationComposer.vue";
import SpaceConversationHeader from "./SpaceConversationHeader.vue";
import SpaceConversationMessages from "./SpaceConversationMessages.vue";
import SpaceConversationRunStatus from "./SpaceConversationRunStatus.vue";

const session = useConversationSessionStore();
const composer = useTemplateRef<InstanceType<typeof SpaceConversationComposer>>("composer");
const { t } = useI18n();
// Three different things were one "busy" flag: a conversation that cannot take a question yet, an
// answer being produced, and a run that exists and can therefore be stopped.
const composerDisabled = computed(
  () =>
    session.restoringDraft ||
    Boolean(session.restorationError) ||
    session.extensionsLoading ||
    Boolean(session.extensionsError),
);
const running = computed(() => session.loading || Boolean(session.activeRunId));
const producing = computed(
  () =>
    running.value &&
    (!session.visibleTaskState ||
      session.visibleTaskState === "queued" ||
      session.visibleTaskState === "running" ||
      (session.visibleTaskState === "waiting" && session.visibleWaitReason === "executor")),
);
const composerError = computed(() => {
  if (session.restorationError) return session.restorationError;
  if (session.draftError) return session.draftError;
  if (session.reacquireFileNames.length)
    return t("spaces.conversation.uploads.reacquire", { names: session.reacquireFileNames.join(", ") });
  return session.attachmentError;
});

async function applySuggestion(value: string) {
  session.prompt = value;
  await nextTick();
  composer.value?.focus();
}

async function previewAttachment(key: string) {
  try {
    await session.previewAttachment(key);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}
</script>

<template>
  <section class="flex h-full min-h-0 min-w-0 flex-col bg-surface-panel">
    <SpaceConversationHeader />
    <button
      v-if="
        session.activeRunId &&
        session.activeConversationId &&
        session.activeConversationId !== session.selectedConversationId
      "
      type="button"
      class="mx-3 mt-2 flex shrink-0 items-center gap-2 rounded-lg border border-stroke-divider/80 bg-surface-raised px-2.5 py-1.5 text-left text-caption text-fg-secondary transition-colors duration-100 hover:border-stroke-default-hover hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
      @click="session.selectConversation(session.activeConversationId)"
    >
      <UiSpinner size="sm" />
      <span class="min-w-0 flex-1 truncate">{{
        $t("spaces.conversation.conversations.returnToRunning")
      }}</span>
      <CornerUpLeft class="size-3.5 shrink-0 text-fg-tertiary" />
    </button>
    <div class="min-h-0 min-w-0 flex-1">
      <SpaceConversationMessages
        class="h-full"
        :scroll-key="session.selectedConversationId"
        :saved-scroll-top="session.scrollTopByConversationId[session.selectedConversationId]"
        :messages="session.messages"
        :events-by-run-id="session.eventsByRunId"
        :active-run-id="session.activeRunId"
        :busy="session.loading"
        :producing="producing"
        :preparing="session.loading && !session.activeRunId"
        :live="session.visibleLive"
        :has-older="session.hasOlder"
        :loading-older="session.loadingOlder"
        @load-older="session.loadOlder"
        @branch="session.branchFrom"
        @regenerate="session.regenerate"
        @scroll-position="session.rememberScroll"
        @suggest="applySuggestion"
        @quote="applySuggestion(quotedFollowUp($event, session.prompt))"
        @inspect-activity="session.inspectActivity"
      />
    </div>
    <ConversationToolInterventions
      v-if="session.activeRunId && session.visibleTaskState === 'waiting'"
      :run-id="session.activeRunId"
      :revision="session.activityRevision"
    />
    <SpaceConversationRunStatus
      :state="session.visibleTaskState"
      :wait-reason="session.visibleWaitReason"
      :error="session.visibleError || session.activityError"
      @resume="session.resume"
      @retry-model="session.retryModel"
    />
    <SpaceConversationComposer
      ref="composer"
      v-model="session.prompt"
      v-model:target="session.target"
      :attachments="session.visibleAttachments"
      :attachment-candidates="session.attachmentCandidates"
      :attachment-error="composerError"
      :picking-files="session.pickingFiles"
      :unresolved-file-names="session.reacquireFileNames"
      :running="running"
      :cancellable="Boolean(session.activeRunId)"
      :disabled="composerDisabled"
      :can-pause="session.canPause"
      @add-selection="session.addSelectedAttachments"
      @add-attachment="(id) => session.addNodeAttachments([id])"
      @choose-files="session.chooseFiles"
      @drop-files="session.chooseFiles($event)"
      @discard-file-reference="session.discardFileReference"
      @remove-attachment="session.removeAttachment"
      @preview-attachment="previewAttachment"
      @send="session.send"
      @cancel="session.cancel"
      @pause="session.pause"
    >
      <template #extensions>
        <AgentExtensionPicker
          v-model="session.extensionSelection"
          :availability="session.extensionAvailability"
          :disabled="composerDisabled || running"
          :error="session.extensionsError"
        />
      </template>
    </SpaceConversationComposer>
  </section>
</template>
