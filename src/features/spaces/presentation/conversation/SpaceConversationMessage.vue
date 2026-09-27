<script setup lang="ts">
import { MoreHorizontal } from "@lucide/vue";
import { writeText } from "tauri-plugin-clipboard-api";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { z } from "zod";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiMarkdown from "../../../../shared/ui/UiMarkdown.vue";
import type { AssistantMessage } from "../../conversation/conversation-ipc";
import { groundedCitationSchema } from "../../knowledge/model/schema/grounded-citation";
import { useSpaceStore } from "../../session/space-store";
import { captureActionLabel } from "../workflows/workflow-capture-labels";
import SpaceConversationMessageAttachments from "./SpaceConversationMessageAttachments.vue";
import SpaceConversationSearchMatches from "./SpaceConversationSearchMatches.vue";

const props = defineProps<{ message: AssistantMessage; streamedText?: string }>();
const emit = defineEmits<{ branch: []; regenerate: [] }>();
const { t } = useI18n();
const store = useSpaceStore();
const copyFailed = shallowRef(false);
const messageText = computed(() =>
  props.message.content.parts
    .filter((part) => part.type === "text")
    .map((part) => part.text)
    .join("\n\n"),
);
const citations = computed(() => groundedCitationSchema.array().parse(props.message.content.citations));
const actions = computed(() => [
  ...(messageText.value
    ? [{ label: t("spaces.assistant.messages.copy"), onSelect: () => void copyMessage() }]
    : []),
  { label: t("spaces.assistant.conversations.branch"), onSelect: () => emit("branch") },
  { label: t("spaces.assistant.regenerate"), onSelect: () => emit("regenerate") },
]);

async function copyMessage() {
  try {
    await writeText(messageText.value);
    copyFailed.value = false;
  } catch {
    copyFailed.value = true;
  }
}

function actionName(id: string) {
  return captureActionLabel(id, t);
}

/** Run results arrive as generated JSON values, so the created-node receipts are untrusted shapes. */
const createdNodeSchema = z.discriminatedUnion("type", [
  z.object({ type: z.literal("accepted_run"), nodeId: z.string() }),
  z.object({ type: z.literal("committed_receipt"), receipt: z.object({ nodeId: z.string().optional() }) }),
]);

function createdNodeId(creationReply: unknown): string | undefined {
  const created = createdNodeSchema.safeParse(creationReply);
  if (!created.success) return undefined;
  return created.data.type === "accepted_run" ? created.data.nodeId : created.data.receipt.nodeId;
}
</script>

<template>
  <div
    class="group flex min-w-0 flex-col gap-2"
    :class="message.role === 'user' ? 'items-end' : 'items-start'"
  >
    <span v-if="message.role === 'assistant'" class="text-caption font-semibold text-fg-secondary">
      {{ t("spaces.assistant.role.assistant") }}
    </span>
    <div
      class="max-w-full min-w-0 space-y-2 text-body leading-relaxed text-fg-primary"
      :class="message.role === 'user' ? 'w-fit rounded-panel bg-surface-control px-3 py-2.5' : 'w-full'"
    >
      <template v-for="(part, index) in message.content.parts" :key="index">
        <UiMarkdown
          v-if="part.type === 'text' && !streamedText"
          :content="part.text"
          class="wrap-break-word"
        />
        <p v-else-if="part.type === 'binary'" class="text-caption text-fg-secondary">
          {{ part.name || part.mediaType }}
        </p>
        <SpaceConversationSearchMatches
          v-else-if="part.type === 'tool_result' && part.name === 'search-knowledge'"
          :value="part.result"
        />
        <p v-else-if="part.type === 'tool_call'" class="text-caption text-fg-secondary">
          {{ t("spaces.assistant.actions.working") }}
        </p>
        <div
          v-else-if="part.type === 'tool_result'"
          class="rounded-md bg-surface-control px-2.5 py-2 text-caption text-fg-secondary"
        >
          <span>{{ t("spaces.assistant.actions.completed", { name: actionName(part.name) }) }}</span>
          <button
            v-if="createdNodeId(part.result)"
            type="button"
            class="ml-2 font-medium text-fg-accent hover:underline"
            @click="store.requestReveal([createdNodeId(part.result)!], 'focus')"
          >
            {{ t("spaces.assistant.actions.viewCreated") }}
          </button>
        </div>
      </template>
      <p v-if="streamedText" class="wrap-break-word whitespace-pre-wrap">{{ streamedText }}</p>
    </div>
    <SpaceConversationMessageAttachments
      v-if="message.role === 'user' && message.content.attachments?.length"
      :attachments="message.content.attachments"
    />
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
    <UiDropdownMenu v-if="message.role === 'assistant'" :items="actions">
      <template #trigger>
        <UiIconButton :label="t('spaces.assistant.messages.actions')" size="sm" class="text-fg-tertiary">
          <MoreHorizontal class="size-3.5" />
        </UiIconButton>
      </template>
    </UiDropdownMenu>
  </div>
</template>
