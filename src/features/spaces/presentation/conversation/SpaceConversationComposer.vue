<script setup lang="ts">
import { ArrowUp, CircleStop, FileText, Link2, X } from "@lucide/vue";
import { computed, ref } from "vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { ConversationTarget } from "../../conversation/conversation-target";
import SpaceConversationAttachmentPicker from "./SpaceConversationAttachmentPicker.vue";
import SpaceConversationTargetField from "./SpaceConversationTargetField.vue";

const prompt = defineModel<string>({ required: true });
const target = defineModel<ConversationTarget | undefined>("target");
const targetAvailable = ref(false);
const props = defineProps<{
  busy: boolean;
  attachments: readonly { key: string; kind: "node" | "file"; title: string }[];
  attachmentCandidates: readonly { nodeId: string; title: string; selected: boolean; attached: boolean }[];
  attachmentError?: string | undefined;
  pickingFiles: boolean;
}>();
const emit = defineEmits<{
  send: [];
  cancel: [];
  addSelection: [];
  addAttachment: [nodeId: string];
  chooseFiles: [];
  removeAttachment: [key: string];
}>();
const canSend = computed(
  () =>
    Boolean(prompt.value.trim() && target.value && targetAvailable.value) &&
    !props.busy &&
    !props.pickingFiles,
);

function submit(event?: KeyboardEvent) {
  if (event?.isComposing || event?.keyCode === 229) return;
  event?.preventDefault();
  if (canSend.value) emit("send");
}
</script>

<template>
  <footer class="min-w-0 shrink-0 border-t border-divider bg-panel p-3">
    <div class="min-w-0 rounded-panel border border-border bg-raised focus-within:border-focus-ring">
      <UiTextarea
        v-model="prompt"
        :rows="3"
        class="max-h-48 min-h-24 resize-none rounded-b-none border-0 bg-transparent px-3 py-3"
        :placeholder="$t('spaces.assistant.builtInPlaceholder')"
        @keydown.enter.exact="submit"
        @keydown.meta.enter="submit"
        @keydown.ctrl.enter="submit"
      />
      <div v-if="attachments.length" class="flex min-w-0 flex-wrap gap-1.5 px-3 pb-2">
        <span
          v-for="attachment in attachments"
          :key="attachment.key"
          class="inline-flex max-w-full items-center gap-1.5 rounded-control border border-divider bg-control px-2 py-1 text-caption"
        >
          <Link2
            v-if="attachment.kind === 'node'"
            class="size-(--knovum-icon-size-xs) shrink-0 text-accent"
          />
          <FileText v-else class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-secondary" />
          <span class="shrink-0 text-foreground-tertiary">{{
            $t(
              attachment.kind === "node"
                ? "spaces.assistant.canvasNodeShort"
                : "spaces.assistant.localFileShort",
            )
          }}</span>
          <span class="truncate">{{ attachment.title }}</span>
          <button
            type="button"
            class="shrink-0 rounded-control text-foreground-secondary hover:text-foreground focus-visible:outline-2 focus-visible:outline-focus-ring"
            :aria-label="$t('spaces.assistant.removeAttachedResource', { name: attachment.title })"
            @click="emit('removeAttachment', attachment.key)"
          >
            <X class="size-(--knovum-icon-size-xs)" />
          </button>
        </span>
      </div>
      <div class="flex min-w-0 items-center gap-1 px-2 pb-2">
        <SpaceConversationAttachmentPicker
          :candidates="attachmentCandidates"
          :picking-files="pickingFiles"
          @add-selection="emit('addSelection')"
          @add-attachment="emit('addAttachment', $event)"
          @choose-files="emit('chooseFiles')"
        />
        <SpaceConversationTargetField
          v-model="target"
          v-model:available="targetAvailable"
          class="min-w-0 flex-1"
        />
        <UiIconButton v-if="busy" :label="$t('spaces.assistant.stop')" size="sm" @click="emit('cancel')">
          <CircleStop class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
        <UiIconButton
          v-else
          :label="$t('spaces.assistant.send')"
          :shortcut="$t('spaces.assistant.sendShortcutHint')"
          size="sm"
          class="bg-accent text-on-accent hover:bg-accent-hover hover:text-on-accent disabled:bg-control disabled:text-foreground-secondary"
          :disabled="!canSend"
          @click="submit()"
        >
          <ArrowUp class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </div>
    </div>
    <p v-if="attachmentError" role="alert" class="pt-1.5 text-caption text-danger">
      {{ attachmentError.startsWith("spaces.assistant.") ? $t(attachmentError) : attachmentError }}
    </p>
  </footer>
</template>
