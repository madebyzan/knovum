<script setup lang="ts">
import { StickyNote } from "@lucide/vue";
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiDragGhost from "../../../../shared/ui/UiDragGhost.vue";
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
import { useAnswerPassage } from "./use-answer-passage";

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
const messageText = computed(() =>
  props.message.content.parts
    .filter((part) => part.type === "text")
    .map((part) => part.text)
    .join("\n\n"),
);
const citations = computed(() => groundedCitationSchema.array().parse(props.message.content.citations));
const capture = useAnswerCapture(citations);
const passage = useAnswerPassage(body, capture, (text) => emit("quote", text));
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
      @mousedown="message.role === 'assistant' && passage.press($event)"
      class="relative min-w-0 space-y-2 text-body leading-relaxed text-fg-primary"
      :class="
        message.role === 'user'
          ? 'w-fit max-w-[80%] rounded-panel bg-surface-control px-3.5 py-2.5'
          : 'w-full'
      "
    >
      <!-- First, not last: `space-y` spaces every child but the last, so a bar appended at the end
           would give the final paragraph a margin and push what follows down while text is selected. -->
      <SpaceConversationSelectionBar
        v-if="message.role === 'assistant' && passage.anchor.value && !passage.drag.value"
        class="absolute z-20 -translate-x-1/2 -translate-y-[calc(100%+6px)]"
        :style="{ left: `${passage.anchor.value.left}px`, top: `${passage.anchor.value.top}px` }"
        :can-save="capture.canSave.value"
        :append-title="capture.appendTarget.value?.title"
        @save="passage.act('save')"
        @append="passage.act('append')"
        @follow-up="passage.act('followUp')"
        @copy="passage.act('copy')"
      />
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
    <UiDragGhost
      v-if="passage.drag.value"
      :x="passage.drag.value.x"
      :y="passage.drag.value.y"
      :droppable="passage.drag.value.target !== undefined"
    >
      <StickyNote class="size-3.5 shrink-0" />
      <span class="truncate">{{ passage.drag.value.item.text }}</span>
    </UiDragGhost>
  </div>
</template>
