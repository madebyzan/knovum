<script setup lang="ts">
import { ArrowUp, CircleStop, X } from "@lucide/vue";
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
  attachments: readonly { nodeId: string; title: string }[];
  attachmentCandidates: readonly { nodeId: string; title: string; selected: boolean; attached: boolean }[];
}>();
const emit = defineEmits<{
  send: [];
  cancel: [];
  addSelection: [];
  addAttachment: [nodeId: string];
  removeAttachment: [nodeId: string];
}>();
const canSend = computed(
  () => Boolean(prompt.value.trim() && target.value && targetAvailable.value) && !props.busy,
);

function submit(event?: KeyboardEvent) {
  if (event?.isComposing || event?.keyCode === 229) return;
  event?.preventDefault();
  if (canSend.value) emit("send");
}
</script>

<template>
  <footer class="min-w-0 shrink-0 border-t border-divider bg-panel p-2.5">
    <div
      class="min-w-0 overflow-hidden rounded-panel border border-border bg-raised focus-within:border-focus-ring"
    >
      <div v-if="attachments.length" class="flex min-w-0 flex-wrap gap-1 border-b border-divider px-2 pt-2">
        <span
          v-for="attachment in attachments"
          :key="attachment.nodeId"
          class="inline-flex max-w-full items-center gap-1 rounded-control bg-control px-2 py-1 text-caption"
        >
          <span class="truncate">{{ attachment.title }}</span>
          <button
            type="button"
            class="shrink-0 rounded-control text-foreground-secondary hover:text-foreground focus-visible:outline-2 focus-visible:outline-focus-ring"
            :aria-label="$t('spaces.assistant.removeAttachedResource', { name: attachment.title })"
            @click="emit('removeAttachment', attachment.nodeId)"
          >
            <X class="size-(--knovum-icon-size-xs)" />
          </button>
        </span>
      </div>
      <UiTextarea
        v-model="prompt"
        :rows="3"
        class="max-h-48 min-h-20 resize-none rounded-b-none border-0 bg-transparent px-3 py-2.5"
        :placeholder="$t('spaces.assistant.builtInPlaceholder')"
        @keydown.enter.exact="submit"
        @keydown.meta.enter="submit"
        @keydown.ctrl.enter="submit"
      />
      <div class="flex min-w-0 items-center gap-1 px-2 pb-2">
        <SpaceConversationAttachmentPicker
          :candidates="attachmentCandidates"
          @add-selection="emit('addSelection')"
          @add-attachment="emit('addAttachment', $event)"
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
          :disabled="!canSend"
          @click="submit()"
        >
          <ArrowUp class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </div>
    </div>
  </footer>
</template>
