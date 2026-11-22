<script setup lang="ts">
import { ArrowUp, FileUp, Pause, Square } from "@lucide/vue";
import { useTextareaAutosize } from "@vueuse/core";
import { computed, ref, useTemplateRef } from "vue";

import { isComposing } from "../../../../foundation/dom/composition";
import type { TaskTarget } from "../../../../shared/contracts/generated/TaskTarget";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type {
  ConversationAttachmentCandidatePresentation,
  ConversationAttachmentPresentation,
} from "../conversation-attachment-presentation";
import SpaceConversationAttachmentPicker from "./SpaceConversationAttachmentPicker.vue";
import SpaceConversationAttachmentTray from "./SpaceConversationAttachmentTray.vue";
import SpaceConversationTargetField from "./SpaceConversationTargetField.vue";
import { useConversationFileDrop } from "./use-conversation-file-drop";

const prompt = defineModel<string>({ required: true });
const target = defineModel<TaskTarget | undefined>("target");
const targetAvailable = ref(false);
const props = defineProps<{
  /** An answer is being produced: the send button becomes the way to stop it. */
  running: boolean;
  /** The run exists and can be stopped; a request that has not been accepted yet cannot. */
  cancellable: boolean;
  /** This conversation cannot accept a question yet — its draft, extensions or Space are not ready. */
  disabled: boolean;
  canPause: boolean;
  attachments: readonly ConversationAttachmentPresentation[];
  attachmentCandidates: readonly ConversationAttachmentCandidatePresentation[];
  attachmentError?: string | undefined;
  pickingFiles: boolean;
  unresolvedFileNames: readonly string[];
}>();
const emit = defineEmits<{
  send: [];
  cancel: [];
  pause: [];
  addSelection: [];
  addAttachment: [nodeId: string];
  chooseFiles: [];
  dropFiles: [paths: string[]];
  discardFileReference: [name: string];
  removeAttachment: [key: string];
  previewAttachment: [key: string];
}>();
const composerRoot = useTemplateRef<HTMLElement>("composerRoot");
const textarea = useTemplateRef<HTMLTextAreaElement>("textarea");
useTextareaAutosize({ element: textarea, input: prompt, maxHeight: 192 });
const { hovering: fileHover } = useConversationFileDrop(composerRoot, {
  blocked: () => props.pickingFiles,
  drop: (paths) => emit("dropFiles", paths),
});
const selectedUnattached = computed(() =>
  props.attachmentCandidates.filter(({ selected, attached }) => selected && !attached),
);
const canSend = computed(
  () =>
    Boolean(prompt.value.trim() && target.value && targetAvailable.value) &&
    !props.running &&
    !props.disabled &&
    !props.pickingFiles &&
    !props.unresolvedFileNames.length,
);
defineExpose({
  /** Focuses the prompt with the caret at the end of the current draft. */
  focus() {
    const input = textarea.value;
    if (!input) return;
    input.focus();
    input.setSelectionRange(prompt.value.length, prompt.value.length);
  },
});

function submit(event?: KeyboardEvent) {
  if (event && isComposing(event)) return;
  event?.preventDefault();
  if (canSend.value) emit("send");
}
</script>

<template>
  <footer class="min-w-0 shrink-0 bg-surface-panel px-3 pt-1 pb-3">
    <div
      ref="composerRoot"
      class="relative min-w-0 rounded-panel border border-stroke-default bg-surface-raised shadow-control transition-colors duration-150 focus-within:border-stroke-strong"
    >
      <div
        v-if="fileHover"
        class="pointer-events-none absolute inset-0 z-10 flex items-center justify-center gap-2 rounded-panel border-2 border-dashed border-stroke-focus bg-surface-raised/95 text-caption font-medium text-fg-accent"
      >
        <FileUp class="size-4" :stroke-width="1.6" />
        {{ $t("spaces.conversation.attachments.dropFiles") }}
      </div>
      <SpaceConversationAttachmentTray
        :attachments="attachments"
        :unresolved-file-names="unresolvedFileNames"
        :selected-candidates="selectedUnattached"
        :show-selection-hint="!running"
        @add-selection="emit('addSelection')"
        @discard-reference="emit('discardFileReference', $event)"
        @remove="emit('removeAttachment', $event)"
        @preview="emit('previewAttachment', $event)"
      />
      <textarea
        ref="textarea"
        v-model="prompt"
        rows="1"
        class="block max-h-48 min-h-[52px] w-full resize-none overflow-y-auto border-0 bg-transparent px-3.5 pt-3 pb-1.5 text-body leading-relaxed text-fg-primary outline-none placeholder:text-fg-tertiary"
        :aria-label="$t('spaces.conversation.input.placeholder')"
        :placeholder="$t('spaces.conversation.input.placeholder')"
        @keydown.enter.exact="submit"
        @keydown.meta.enter="submit"
        @keydown.ctrl.enter="submit"
      />
      <div class="flex min-w-0 items-center gap-1.5 px-2.5 pt-0.5 pb-2.5">
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
        <slot name="extensions" />
        <!-- One place to act on the answer in flight: pause it if it can be resumed, or stop it. -->
        <UiIconButton
          v-if="running && canPause"
          :label="$t('common.pause')"
          tooltip
          size="sm"
          @click="emit('pause')"
        >
          <Pause class="size-3.5" />
        </UiIconButton>
        <UiIconButton
          v-if="running"
          :label="$t('spaces.conversation.stop')"
          tooltip
          size="sm"
          tone="primary"
          :disabled="!cancellable"
          @click="emit('cancel')"
        >
          <Square class="size-2.5 fill-current" />
        </UiIconButton>
        <UiIconButton
          v-else
          :label="$t('spaces.conversation.input.send')"
          :shortcut="$t('spaces.conversation.input.shortcutHint')"
          size="sm"
          :tone="canSend ? 'primary' : 'neutral'"
          :class="!canSend && 'rounded-full bg-surface-control text-fg-tertiary'"
          :disabled="!canSend"
          @click="submit()"
        >
          <ArrowUp class="size-4" :stroke-width="2.1" />
        </UiIconButton>
      </div>
    </div>
    <p v-if="attachmentError" role="alert" class="pt-1.5 text-caption text-fg-danger">
      {{ attachmentError }}
    </p>
  </footer>
</template>
