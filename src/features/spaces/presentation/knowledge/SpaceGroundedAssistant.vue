<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceKnowledgeAssistant } from "../../knowledge/assistant/use-assistant";
import { useAssistantScroll } from "../../knowledge/assistant/use-assistant-scroll";
import SpaceKnowledgeAssistantComposer from "./SpaceKnowledgeAssistantComposer.vue";
import SpaceKnowledgeAssistantEmpty from "./SpaceKnowledgeAssistantEmpty.vue";
import SpaceKnowledgeConversation from "./SpaceKnowledgeConversation.vue";
import SpaceKnowledgeIndexStatus from "./SpaceKnowledgeIndexStatus.vue";
import SpaceKnowledgeThreadMenu from "./SpaceKnowledgeThreadMenu.vue";
import SpaceKnowledgeThreadPicker from "./SpaceKnowledgeThreadPicker.vue";

const { t } = useI18n();
const {
  store,
  scope,
  question,
  executionTarget,
  busy,
  messages,
  pendingQuestion,
  streamedAnswer,
  answerStage,
  semantic,
  activeThread,
  activeScope,
  scopeCounts,
  scopeTitles,
  threadOptions,
  selectedThread,
  askKnowledge,
  cancelAsk,
  renameThread,
  deleteThread,
  saveAsKnowledge,
} = useSpaceKnowledgeAssistant();
const messageViewport = useTemplateRef<HTMLElement>("messageViewport");
const { onScroll } = useAssistantScroll({
  viewport: messageViewport,
  busy,
  pendingQuestion,
  streamedAnswer,
  messageCount: computed(() => messages.value.length),
});

function useSuggestedPrompt(prompt: string) {
  question.value = prompt;
}
</script>

<template>
  <section class="flex h-full min-h-0 flex-col bg-panel">
    <header class="shrink-0 px-3 py-2.5">
      <div class="flex min-w-0 items-center gap-1">
        <SpaceKnowledgeThreadPicker v-model="selectedThread" :options="threadOptions" :disabled="busy" />
        <SpaceKnowledgeThreadMenu
          v-if="activeThread"
          :title="activeThread.title"
          :disabled="busy"
          @rename="renameThread"
          @delete="deleteThread"
        />
      </div>
      <p v-if="activeScope === 'all-spaces'" class="mt-1.5 px-1 text-micro text-foreground-tertiary">
        {{ t("spaces.assistant.allSpacesReadOnly") }}
      </p>
      <SpaceKnowledgeIndexStatus
        v-if="store.activeSpaceId && activeScope !== 'all-spaces'"
        class="mt-1.5"
        :space-id="store.activeSpaceId"
      />
    </header>
    <div
      ref="messageViewport"
      class="min-h-0 flex-1 overflow-y-auto border-t border-divider"
      @scroll.passive="onScroll"
    >
      <SpaceKnowledgeAssistantEmpty
        v-if="messages.length === 0 && !busy"
        :busy="busy"
        @select="useSuggestedPrompt"
      />
      <SpaceKnowledgeConversation
        v-else
        class="px-3 py-3"
        :messages="messages"
        :pending-question="pendingQuestion"
        :streamed-answer="streamedAnswer"
        :answer-stage="answerStage"
        :busy="busy"
        :can-save-as-knowledge="activeScope !== 'all-spaces'"
        @save-as-knowledge="saveAsKnowledge"
      />
      <p v-if="semantic === 'unavailable'" class="px-3 py-2 text-micro text-warning">
        {{ t("spaces.processing.semanticUnavailable") }}
      </p>
    </div>
    <SpaceKnowledgeAssistantComposer
      v-model="question"
      v-model:execution-target="executionTarget"
      v-model:scope="scope"
      :busy="busy"
      :scope-locked="Boolean(activeThread)"
      :selection-count="scopeCounts.selection"
      :section-count="scopeCounts.section"
      :space-count="scopeCounts.space"
      :scope-titles="scopeTitles"
      @ask="askKnowledge()"
      @cancel="cancelAsk"
    />
  </section>
</template>
