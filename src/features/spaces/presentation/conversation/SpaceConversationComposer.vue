<script setup lang="ts">
import { ArrowUp, CircleStop } from "@lucide/vue";
import { computed, ref, useTemplateRef } from "vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { ConversationTarget } from "../../conversation/conversation-target";
import SpaceConversationAttachment from "./SpaceConversationAttachment.vue";
import SpaceConversationAttachmentPicker from "./SpaceConversationAttachmentPicker.vue";
import SpaceConversationTargetField from "./SpaceConversationTargetField.vue";

const prompt = defineModel<string>({ required: true });
const target = defineModel<ConversationTarget | undefined>("target");
const targetAvailable = ref(false);
const props = defineProps<{
  busy: boolean;
  attachments: readonly { key: string; kind: "node" | "file"; title: string; previewSource: string }[];
  attachmentCandidates: readonly {
    nodeId: string;
    title: string;
    previewSource: string;
    selected: boolean;
    attached: boolean;
  }[];
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
const input = useTemplateRef<InstanceType<typeof UiTextarea>>("input");
defineExpose({ focus: () => input.value?.focus() });

function submit(event?: KeyboardEvent) {
  if (event?.isComposing || event?.keyCode === 229) return;
  event?.preventDefault();
  if (canSend.value) emit("send");
}
</script>

<template>
  <footer class="min-w-0 shrink-0 border-t border-stroke-divider bg-surface-panel p-3">
    <div
      class="min-w-0 rounded-panel border border-stroke-default bg-surface-raised focus-within:border-stroke-focus"
    >
      <UiTextarea
        v-if="!busy"
        ref="input"
        v-model="prompt"
        :rows="3"
        class="max-h-48 min-h-24 resize-none rounded-b-none border-0 bg-transparent px-3 py-3"
        :placeholder="$t('spaces.assistant.input.placeholder')"
        @keydown.enter.exact="submit"
        @keydown.meta.enter="submit"
        @keydown.ctrl.enter="submit"
      />
      <div v-else class="min-h-24 px-3 py-3 text-body text-fg-tertiary">
        {{ $t("spaces.assistant.input.placeholder") }}
      </div>
      <div v-if="attachments.length" class="flex min-w-0 flex-wrap gap-1.5 px-3 pb-2">
        <SpaceConversationAttachment
          v-for="attachment in attachments"
          :key="attachment.key"
          :attachment="attachment"
          @remove="emit('removeAttachment', $event)"
        />
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
          <CircleStop class="size-3.5" />
        </UiIconButton>
        <UiIconButton
          v-else
          :label="$t('spaces.assistant.input.send')"
          :shortcut="$t('spaces.assistant.input.shortcutHint')"
          size="sm"
          class="bg-surface-accent text-fg-on-accent hover:bg-surface-accent-hover hover:text-fg-on-accent disabled:bg-surface-control disabled:text-fg-secondary"
          :disabled="!canSend"
          @click="submit()"
        >
          <ArrowUp class="size-3.5" />
        </UiIconButton>
      </div>
    </div>
    <p v-if="attachmentError" role="alert" class="pt-1.5 text-caption text-fg-danger">
      {{ attachmentError.startsWith("spaces.assistant.") ? $t(attachmentError) : attachmentError }}
    </p>
  </footer>
</template>
