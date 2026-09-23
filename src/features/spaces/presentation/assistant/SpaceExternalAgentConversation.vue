<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { useExternalAgentConversation } from "../../agent-creation/use-external-agent-conversation";
import SpaceExternalAttachments from "./SpaceExternalAttachments.vue";
import SpaceExternalConversationMessages from "./SpaceExternalConversationMessages.vue";
import SpaceExternalConversationToolbar from "./SpaceExternalConversationToolbar.vue";
import SpaceExternalSessionOptions from "./SpaceExternalSessionOptions.vue";

const { t } = useI18n();
const {
  agentOptions,
  attachmentState,
  busy,
  close,
  conversationOptions,
  error,
  messages,
  prompt,
  remove,
  select,
  selected,
  selectedId,
  send,
  setSessionMode,
  setSessionOption,
  start,
} = useExternalAgentConversation();
</script>

<template>
  <section class="flex h-full min-h-0 flex-col bg-panel">
    <header class="shrink-0 space-y-2 border-b border-divider p-2.5">
      <SpaceExternalConversationToolbar
        :selected-id="selectedId"
        :selected-state="selected?.state"
        :conversation-options="conversationOptions"
        :agent-options="agentOptions"
        :busy="busy"
        @select="select"
        @start="start"
        @close="close"
        @remove="remove"
      />
      <p v-if="selected?.state === 'disconnected'" class="text-caption text-warning">
        {{ t("spaces.assistant.externalConversationDisconnected") }}
      </p>
      <p v-if="selected?.error" class="text-meta line-clamp-3 text-danger">{{ selected.error }}</p>
      <SpaceExternalSessionOptions
        v-if="selected"
        :capabilities="selected.sessionCapabilities"
        :disabled="busy"
        @change="setSessionOption"
        @mode="setSessionMode"
      />
    </header>
    <SpaceExternalConversationMessages :messages="messages" />
    <p v-if="error" class="text-meta px-3 py-2 text-danger">{{ error }}</p>
    <form class="shrink-0 space-y-2 border-t border-divider p-2.5" @submit.prevent="send">
      <UiTextarea
        v-model="prompt"
        :rows="3"
        :placeholder="t('spaces.assistant.externalConversationPrompt')"
      />
      <SpaceExternalAttachments
        :attachments="attachmentState.attachments.value"
        :busy="busy"
        @choose="attachmentState.choose"
        @remove="attachmentState.remove"
      />
      <div class="flex items-center justify-between gap-2">
        <p class="text-caption text-foreground-tertiary">
          {{ t("spaces.assistant.externalPermissionHint") }}
        </p>
        <UiButton
          type="submit"
          variant="primary"
          :disabled="busy || selected?.state !== 'connected' || !prompt.trim()"
        >
          {{ t("common.send") }}
        </UiButton>
      </div>
    </form>
  </section>
</template>
