<script setup lang="ts">
import { StickyNote } from "@lucide/vue";
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiCollapsibleClamp from "../../../../shared/ui/UiCollapsibleClamp.vue";
import UiCopyButton from "../../../../shared/ui/UiCopyButton.vue";
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
import SpaceCitationPreview from "./SpaceCitationPreview.vue";
import SpaceConversationCitations from "./SpaceConversationCitations.vue";
import SpaceConversationMessageActions from "./SpaceConversationMessageActions.vue";
import SpaceConversationMessageAttachments from "./SpaceConversationMessageAttachments.vue";
import SpaceConversationOutputs from "./SpaceConversationOutputs.vue";
import SpaceConversationSelectionBar from "./SpaceConversationSelectionBar.vue";
import { useAnswerPassage } from "./use-answer-passage";
import { useCitationHover } from "./use-citation-hover";

const props = defineProps<{ message: ConversationMessage; actions?: boolean; busy: boolean }>();
const emit = defineEmits<{ branch: []; regenerate: []; quote: [text: string] }>();
const { t } = useI18n();
const store = useSpaceStore();
const body = useTemplateRef<HTMLElement>("body");
const messageText = computed(() =>
  props.message.content.parts
    .filter((p) => p.type === "text")
    .map((p) => p.text)
    .join("\n\n"),
);
const hasVisibleParts = computed(() =>
  props.message.content.parts.some(
    (p) => (p.type === "text" && p.text.trim().length > 0) || p.type === "binary",
  ),
);
const citations = computed(() => groundedCitationSchema.array().parse(props.message.content.citations));
const capture = useAnswerCapture(citations);
const passage = useAnswerPassage(body, capture, (text) => emit("quote", text));
const citationHover = useCitationHover(citations);
const citationMarkers = computed(() => new Map(citations.value.map((c) => [c.index, c.state === "current"])));
const statusLabel = computed(() =>
  props.message.content.status === "stopped"
    ? t("spaces.conversation.response.stopped")
    : t("spaces.conversation.response.incomplete"),
);

function openCitation(citation: GroundedCitation | undefined) {
  citationHover.dismiss();
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
    <div v-if="message.role === 'user'" class="flex w-full min-w-0 items-end justify-end gap-1">
      <div
        v-if="messageText"
        class="shrink-0 pb-0.5 opacity-0 transition-opacity duration-120 will-change-[opacity] group-focus-within:opacity-100 group-hover:opacity-100"
      >
        <UiCopyButton
          :text="messageText"
          :label="t('spaces.conversation.messages.copy')"
          :copied-label="t('spaces.conversation.messages.copied')"
          :error-message="t('spaces.conversation.messages.copyFailed')"
        />
      </div>
      <div
        v-if="hasVisibleParts"
        ref="body"
        class="w-fit max-w-[88%] min-w-0 rounded-panel bg-surface-control px-3.5 py-2.5 text-body leading-relaxed text-fg-primary ring-1 ring-stroke-divider/45 ring-inset"
      >
        <UiCollapsibleClamp :max-height="128" :content-key="message.id">
          <div class="space-y-2">
            <template v-for="(part, index) in message.content.parts" :key="index">
              <p
                v-if="part.type === 'text' && part.text.trim()"
                class="wrap-break-word whitespace-pre-wrap select-text"
              >
                {{ part.text }}
              </p>
              <p v-else-if="part.type === 'binary'" class="text-caption text-fg-secondary">
                {{ part.name || part.mediaType }}
              </p>
            </template>
          </div>
        </UiCollapsibleClamp>
      </div>
    </div>
    <div
      v-else
      ref="body"
      class="relative w-full min-w-0 space-y-2 text-body leading-relaxed text-fg-primary"
      @mousedown="passage.press($event)"
      @pointerover="citationHover.onBodyPointerOver"
      @pointerout="citationHover.onBodyPointerOut"
    >
      <SpaceConversationSelectionBar
        v-if="passage.anchor.value && !passage.drag.value"
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
          @cite="openCitation(citations.find((c) => c.index === $event))"
          @link="followLink"
        />
        <p v-else-if="part.type === 'binary'" class="text-caption text-fg-secondary">
          {{ part.name || part.mediaType }}
        </p>
      </template>
      <SpaceCitationPreview
        :state="citationHover.active.value"
        @open="openCitation"
        @preview-enter="citationHover.onPreviewPointerEnter"
        @preview-leave="citationHover.onPreviewPointerLeave"
      />
    </div>
    <SpaceConversationOutputs
      v-if="message.role === 'assistant' && message.content.outputs?.length"
      :outputs="message.content.outputs"
    />
    <SpaceConversationCitations v-if="citations.length" :citations="citations" @open="openCitation" />
    <span v-if="message.content.status" class="text-caption-sm text-fg-tertiary">{{ statusLabel }}</span>
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
