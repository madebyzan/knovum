<script setup lang="ts">
import { Check, Copy, GitBranch, RefreshCw } from "@lucide/vue";
import { useTimeoutFn } from "@vueuse/core";
import { writeText } from "tauri-plugin-clipboard-api";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";
import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import {
  groundedCitationSchema,
  type GroundedCitation,
} from "../../knowledge/model/schema/grounded-citation";
import { useSpaceStore } from "../../session/space-store";
import { followConversationLink } from "../conversation-links";
import SpaceConversationCitations from "./SpaceConversationCitations.vue";
import SpaceConversationMessageAttachments from "./SpaceConversationMessageAttachments.vue";
import SpaceConversationOutputs from "./SpaceConversationOutputs.vue";

const props = defineProps<{
  message: ConversationMessage;
  /** Offers copy, branch and regenerate: the answer that ends a turn does, the words before it do not. */
  actions?: boolean;
  busy: boolean;
}>();
const emit = defineEmits<{ branch: []; regenerate: [] }>();
const { t } = useI18n();
const store = useSpaceStore();
const copyFailed = shallowRef(false);
const copied = shallowRef(false);
const { start: resetCopied } = useTimeoutFn(() => (copied.value = false), 1600, { immediate: false });
const messageText = computed(() =>
  props.message.content.parts
    .filter((part) => part.type === "text")
    .map((part) => part.text)
    .join("\n\n"),
);
const citations = computed(() => groundedCitationSchema.array().parse(props.message.content.citations));
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

async function copyMessage() {
  try {
    await writeText(messageText.value);
    copyFailed.value = false;
    copied.value = true;
    resetCopied();
  } catch {
    copyFailed.value = true;
    copied.value = false;
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
      class="min-w-0 space-y-2 text-body leading-relaxed text-fg-primary"
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
    <p v-if="copyFailed" role="alert" class="text-caption text-fg-danger">
      {{ t("spaces.conversation.messages.copyFailed") }}
    </p>
    <div
      v-if="actions && message.role === 'assistant' && messageText"
      class="message-actions flex items-center gap-0.5 opacity-0 transition-opacity duration-100 group-focus-within:opacity-100 group-hover:opacity-100"
    >
      <UiIconButton
        :label="t(copied ? 'spaces.conversation.messages.copied' : 'spaces.conversation.messages.copy')"
        size="sm"
        @click="copyMessage"
      >
        <Check v-if="copied" class="size-3.5 text-fg-accent" />
        <Copy v-else class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.conversation.conversations.branch')"
        tooltip
        size="sm"
        :disabled="busy"
        @click="emit('branch')"
      >
        <GitBranch class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.conversation.regenerate')"
        tooltip
        size="sm"
        :disabled="busy"
        @click="emit('regenerate')"
      >
        <RefreshCw class="size-3.5" />
      </UiIconButton>
    </div>
  </div>
</template>

<style scoped>
@media (hover: none) {
  .message-actions {
    opacity: 1;
  }
}
</style>
