<script setup lang="ts">
import { writeText } from "tauri-plugin-clipboard-api";
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";
import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import {
  groundedCitationSchema,
  type GroundedCitation,
} from "../../knowledge/model/schema/grounded-citation";
import { useSpaceStore } from "../../session/space-store";
import { followConversationLink } from "../conversation-links";
import { useAnswerCapture } from "../use-answer-capture";
import SpaceConversationCitations from "./SpaceConversationCitations.vue";
import SpaceConversationMessageActions from "./SpaceConversationMessageActions.vue";
import SpaceConversationMessageAttachments from "./SpaceConversationMessageAttachments.vue";
import SpaceConversationOutputs from "./SpaceConversationOutputs.vue";
import SpaceConversationSelectionBar from "./SpaceConversationSelectionBar.vue";
import { useAnswerSelection } from "./use-answer-selection";

const props = defineProps<{
  message: ConversationMessage;
  /** Offers copy, branch and regenerate: the answer that ends a turn does, the words before it do not. */
  actions?: boolean;
  busy: boolean;
}>();
const emit = defineEmits<{ branch: []; regenerate: []; quote: [text: string] }>();
const { t } = useI18n();
const store = useSpaceStore();
const body = useTemplateRef<HTMLElement>("body");
const selected = useAnswerSelection(body);
const messageText = computed(() =>
  props.message.content.parts
    .filter((part) => part.type === "text")
    .map((part) => part.text)
    .join("\n\n"),
);
const citations = computed(() => groundedCitationSchema.array().parse(props.message.content.citations));
const capture = useAnswerCapture(citations);
const citationMarkers = computed(
  () => new Map(citations.value.map((citation) => [citation.index, citation.state === "current"])),
);

function openCitation(citation: GroundedCitation | undefined) {
  if (citation?.state !== "current") return;
  store.openSource({
    nodeId: citation.nodeId,
    nodeTitle: citation.nodeTitle,
    chunkId: citation.chunkId,
    locator: citation.locator,
    text: citation.quote,
  });
}

async function followLink(href: string) {
  try {
    await followConversationLink(props.message.id, href);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}

// Each action ends the selection: the passage has been dealt with, and its bar would linger.
async function onSelection(action: "save" | "append" | "followUp" | "copy") {
  const passage = selected.passage.value;
  selected.clear();
  if (!passage) return;
  if (action === "save") await capture.saveSelection(passage);
  else if (action === "append") await capture.appendSelection(passage);
  else if (action === "followUp") emit("quote", passage.text);
  else await writeText(passage.text).catch((error: unknown) => toast.error(getErrorMessage(error)));
}

function dragSelection(event: DragEvent) {
  if (props.message.role === "assistant" && selected.passage.value)
    capture.dragSelection(event, selected.passage.value);
}
</script>

<template>
  <div
    class="group flex min-w-0 flex-col gap-2"
    :class="message.role === 'user' ? 'items-end' : 'items-start'"
  >
    <SpaceConversationMessageAttachments
      v-if="message.role === 'user' && message.content.attachments?.length"
      :attachments="message.content.attachments"
    />
    <div
      ref="body"
      @dragstart="dragSelection"
      class="relative min-w-0 space-y-2 text-body leading-relaxed text-fg-primary"
      :class="
        message.role === 'user'
          ? 'w-fit max-w-[80%] rounded-panel bg-surface-control px-3.5 py-2.5'
          : 'w-full'
      "
    >
      <template v-for="(part, index) in message.content.parts" :key="index">
        <UiMarkdown
          v-if="part.type === 'text' && part.text.trim()"
          :content="part.text"
          :citations="citationMarkers"
          local-links
          class="wrap-break-word"
          @cite="openCitation(citations.find((citation) => citation.index === $event))"
          @link="followLink"
        />
        <p v-else-if="part.type === 'binary'" class="text-caption text-fg-secondary">
          {{ part.name || part.mediaType }}
        </p>
      </template>
      <SpaceConversationSelectionBar
        v-if="message.role === 'assistant' && selected.anchor.value"
        class="absolute z-20 -translate-x-1/2 -translate-y-[calc(100%+6px)]"
        :style="{ left: `${selected.anchor.value.left}px`, top: `${selected.anchor.value.top}px` }"
        :can-save="capture.canSave.value"
        :append-title="capture.appendTarget.value?.title"
        @save="onSelection('save')"
        @append="onSelection('append')"
        @follow-up="onSelection('followUp')"
        @copy="onSelection('copy')"
      />
    </div>
    <SpaceConversationOutputs
      v-if="message.role === 'assistant' && message.content.outputs?.length"
      :outputs="message.content.outputs"
    />
    <SpaceConversationCitations v-if="citations.length" :citations="citations" @open="openCitation" />
    <span v-if="message.content.status" class="text-caption-sm text-fg-tertiary">
      {{
        t(
          message.content.status === "stopped"
            ? "spaces.conversation.response.stopped"
            : "spaces.conversation.response.incomplete",
        )
      }}
    </span>
    <SpaceConversationMessageActions
      v-if="actions && message.role === 'assistant' && messageText"
      :text="messageText"
      :busy="busy"
      :can-save="capture.canSave.value"
      @save="capture.saveAnswer(messageText)"
      @branch="emit('branch')"
      @regenerate="emit('regenerate')"
    />
  </div>
</template>
