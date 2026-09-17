<script setup lang="ts">
import { Copy, LoaderCircle, SquarePlus } from "@lucide/vue";
import { reactive } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import type { GroundedAnswerStage } from "../../knowledge/assistant/grounded-assistant";
import { groundedCitationCurrent } from "../../knowledge/assistant/grounded-citation-actions";
import { findGroundedCitation } from "../../knowledge/model/grounded-citation";
import type { GroundedMessage } from "../../knowledge/model/grounded-message";
import { useSpaceStore } from "../../session/space-store";
import SpaceGroundedCitationList from "./SpaceGroundedCitationList.vue";
import SpaceGroundedMarkdown from "./SpaceGroundedMarkdown.vue";

defineProps<{
  messages: readonly GroundedMessage[];
  pendingQuestion: string;
  streamedAnswer: string;
  answerStage: GroundedAnswerStage | undefined;
  busy: boolean;
  canSaveAsKnowledge: boolean;
}>();
const emit = defineEmits<{ saveAsKnowledge: [message: GroundedMessage] }>();
const { t } = useI18n();
const store = useSpaceStore();
const editorSession = useCanvasEditorSession();
const activeCitations = reactive<Record<string, number>>({});

function highlightCitation(message: GroundedMessage, index: number) {
  activeCitations[message.id] = index;
  const citation = findGroundedCitation(message.citations, index);
  if (citation?.state === "current" && groundedCitationCurrent(store, citation)) {
    editorSession.reveal([citation.nodeId], "focus");
  }
}

async function copyAnswer(content: string) {
  await navigator.clipboard.writeText(content);
}
</script>

<template>
  <div class="space-y-4">
    <template v-for="message in messages" :key="message.id">
      <div v-if="message.role === 'user'" class="border-b border-divider pb-3">
        <p class="text-body leading-relaxed font-medium whitespace-pre-wrap text-foreground">
          {{ message.content }}
        </p>
      </div>
      <article v-else class="border-b border-divider pb-3 last:border-b-0">
        <SpaceGroundedMarkdown
          :content="message.content"
          :citations="message.citations"
          v-bind="
            activeCitations[message.id] === undefined
              ? {}
              : { activeCitationIndex: activeCitations[message.id] }
          "
          @highlight-citation="highlightCitation(message, $event)"
        />
        <SpaceGroundedCitationList
          v-if="message.citations.length"
          :citations="message.citations"
          v-bind="
            activeCitations[message.id] === undefined ? {} : { activeIndex: activeCitations[message.id] }
          "
          @highlight-citation="highlightCitation(message, $event)"
        />
        <div class="mt-1.5 flex items-center gap-0.5">
          <UiButton size="sm" variant="ghost" class="px-2" @click="copyAnswer(message.content)">
            <Copy class="size-(--knovum-icon-size-xs)" />{{ t("common.copy") }}
          </UiButton>
          <UiButton
            v-if="canSaveAsKnowledge"
            size="sm"
            variant="ghost"
            class="px-2"
            @click="emit('saveAsKnowledge', message)"
          >
            <SquarePlus class="size-(--knovum-icon-size-xs)" />{{
              t("spaces.assistant.aiAction.saveAsKnowledge")
            }}
          </UiButton>
        </div>
      </article>
    </template>
    <div v-if="pendingQuestion" class="border-b border-divider pb-3">
      <p class="text-body font-medium whitespace-pre-wrap text-foreground">
        {{ pendingQuestion }}
      </p>
    </div>
    <article v-if="busy" class="border-b border-divider pb-3" aria-live="polite">
      <SpaceGroundedMarkdown v-if="streamedAnswer" :content="streamedAnswer" />
      <div v-else class="flex items-center gap-2 py-1 text-caption text-foreground-tertiary">
        <LoaderCircle class="size-(--knovum-icon-size-xs) animate-spin" />{{
          t(`spaces.assistant.aiStage.${answerStage ?? "retrieving"}`)
        }}
      </div>
    </article>
  </div>
</template>
