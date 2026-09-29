<script setup lang="ts">
import { Check, Copy, GitBranch, RefreshCw } from "@lucide/vue";
import { useTimeoutFn } from "@vueuse/core";
import { writeText } from "tauri-plugin-clipboard-api";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";
import type { AssistantMessage } from "../../conversation/conversation-ipc";
import { groundedCitationSchema } from "../../knowledge/model/schema/grounded-citation";
import { useSpaceStore } from "../../session/space-store";
import SpaceConversationMessageAttachments from "./SpaceConversationMessageAttachments.vue";

const props = defineProps<{ message: AssistantMessage; streamedText?: string; busy: boolean }>();
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
      :class="message.role === 'user' ? 'w-fit max-w-[85%] rounded-panel bg-surface-control px-3 py-2.5' : 'w-full'"
    >
      <template v-for="(part, index) in message.content.parts" :key="index">
        <UiMarkdown
          v-if="part.type === 'text' && part.text.trim() && !streamedText"
          :content="part.text"
          class="wrap-break-word"
        />
        <p v-else-if="part.type === 'binary'" class="text-caption text-fg-secondary">
          {{ part.name || part.mediaType }}
        </p>
      </template>
      <p v-if="streamedText" class="wrap-break-word whitespace-pre-wrap">{{ streamedText }}</p>
    </div>
    <details
      v-if="citations.length"
      class="w-full rounded-md border border-stroke-divider px-2 py-1.5 text-caption"
    >
      <summary class="cursor-pointer text-fg-secondary select-none">
        {{ t("spaces.assistant.sources", { count: citations.length }) }}
      </summary>
      <ul class="mt-2 space-y-1.5">
        <li
          v-for="citation in citations"
          :key="citation.index"
          class="rounded-md bg-surface-control px-2 py-1.5"
        >
          <button
            v-if="citation.state === 'current'"
            type="button"
            class="text-left font-medium text-fg-accent hover:underline"
            @click="
              store.openSource({
                nodeId: citation.nodeId,
                nodeTitle: citation.nodeTitle,
                chunkId: citation.chunkId,
                locator: citation.locator,
                text: citation.quote,
              })
            "
          >
            {{ citation.index }}. {{ citation.nodeTitle }}
          </button>
          <span v-else class="font-medium text-fg-secondary"
            >{{ citation.index }}. {{ citation.nodeTitle }}</span
          >
          <span v-if="citation.state === 'detached'" class="ml-1 text-fg-tertiary">
            · {{ t("spaces.assistant.evidence.stale") }}
          </span>
          <p class="mt-1 line-clamp-3 whitespace-pre-wrap text-fg-secondary">{{ citation.quote }}</p>
        </li>
      </ul>
    </details>
    <span v-if="message.content.status" class="text-caption-sm text-fg-tertiary">
      {{
        t(
          message.content.status === "stopped"
            ? "spaces.assistant.response.stopped"
            : "spaces.assistant.response.incomplete",
        )
      }}
    </span>
    <p v-if="copyFailed" role="alert" class="text-caption text-fg-danger">
      {{ t("spaces.assistant.messages.copyFailed") }}
    </p>
    <div
      v-if="message.role === 'assistant' && messageText"
      class="message-actions flex items-center gap-0.5 opacity-0 transition-opacity duration-100 group-hover:opacity-100 group-focus-within:opacity-100"
    >
      <UiIconButton :label="t(copied ? 'spaces.assistant.messages.copied' : 'spaces.assistant.messages.copy')" size="sm" @click="copyMessage">
        <Check v-if="copied" class="size-3.5 text-fg-accent" />
        <Copy v-else class="size-3.5" />
      </UiIconButton>
      <UiIconButton :label="t('spaces.assistant.conversations.branch')" size="sm" :disabled="busy" @click="emit('branch')">
        <GitBranch class="size-3.5" />
      </UiIconButton>
      <UiIconButton :label="t('spaces.assistant.regenerate')" size="sm" :disabled="busy" @click="emit('regenerate')">
        <RefreshCw class="size-3.5" />
      </UiIconButton>
    </div>
  </div>
</template>

<style scoped>
@media (hover: none) {
  .message-actions { opacity: 1; }
}
</style>
