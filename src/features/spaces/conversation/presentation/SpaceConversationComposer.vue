<script setup lang="ts">
import { ArrowUp, CircleStop, FileUp } from "@lucide/vue";
import { useTextareaAutosize } from "@vueuse/core";
import { computed, onBeforeUnmount, onMounted, ref, shallowRef, useTemplateRef } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { isFileDropInside, listenForFileDrops } from "../../../../foundation/file-drop/events";
import { createAsyncDisposer } from "../../../../foundation/lifecycle/async-disposer";
import type { TaskTarget } from "../../../../shared/contracts/generated/TaskTarget";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type {
  ConversationAttachmentCandidatePresentation,
  ConversationAttachmentPresentation,
} from "../conversation-attachment-presentation";
import SpaceConversationAttachmentPicker from "./SpaceConversationAttachmentPicker.vue";
import SpaceConversationAttachmentTray from "./SpaceConversationAttachmentTray.vue";
import SpaceConversationTargetField from "./SpaceConversationTargetField.vue";

const prompt = defineModel<string>({ required: true });
const target = defineModel<TaskTarget | undefined>("target");
const targetAvailable = ref(false);
const props = defineProps<{
  busy: boolean;
  attachments: readonly ConversationAttachmentPresentation[];
  attachmentCandidates: readonly ConversationAttachmentCandidatePresentation[];
  attachmentError?: string | undefined;
  pickingFiles: boolean;
  unresolvedFileNames: readonly string[];
}>();
const emit = defineEmits<{
  send: [];
  cancel: [];
  addSelection: [];
  addAttachment: [nodeId: string];
  chooseFiles: [];
  dropFiles: [paths: string[]];
  discardFileReference: [name: string];
  removeAttachment: [key: string];
  previewAttachment: [key: string];
}>();
const composerRoot = useTemplateRef<HTMLElement>("composerRoot");
const { textarea } = useTextareaAutosize({ input: prompt, maxHeight: 192 });
const fileHover = shallowRef(false);
const fileDropListener = createAsyncDisposer();
const canSend = computed(
  () =>
    Boolean(prompt.value.trim() && target.value && targetAvailable.value) &&
    !props.busy &&
    !props.pickingFiles &&
    !props.unresolvedFileNames.length,
);
defineExpose({
  focus(selection?: string) {
    const input = textarea.value;
    input?.focus();
    if (!input || !selection) return;
    const start = prompt.value.indexOf(selection);
    if (start >= 0) input.setSelectionRange(start, start + selection.length);
  },
});

function submit(event?: KeyboardEvent) {
  if (event?.isComposing || event?.keyCode === 229) return;
  event?.preventDefault();
  if (canSend.value) emit("send");
}

onMounted(() => {
  void fileDropListener
    .attach(
      listenForFileDrops({
        hover(active, position) {
          fileHover.value = active && !props.pickingFiles && isFileDropInside(composerRoot.value, position);
        },
        drop({ paths, position }) {
          fileHover.value = false;
          if (paths.length && !props.pickingFiles && isFileDropInside(composerRoot.value, position))
            emit("dropFiles", paths);
        },
      }),
    )
    .catch((error) => toast.error(getErrorMessage(error)));
});
onBeforeUnmount(fileDropListener.dispose);
</script>

<template>
  <footer class="min-w-0 shrink-0 bg-surface-panel px-3 pt-1 pb-3">
    <div
      ref="composerRoot"
      class="relative min-w-0 rounded-panel border border-stroke-default bg-surface-raised shadow-control transition-colors focus-within:border-stroke-focus"
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
        @discard-reference="emit('discardFileReference', $event)"
        @remove="emit('removeAttachment', $event)"
        @preview="emit('previewAttachment', $event)"
      />
      <textarea
        ref="textarea"
        v-model="prompt"
        rows="2"
        class="composer-input block max-h-48 min-h-16 w-full resize-none overflow-y-auto border-0 bg-transparent px-3 py-3 text-body text-fg-primary outline-none placeholder:text-fg-tertiary"
        :aria-label="$t('spaces.conversation.input.placeholder')"
        :placeholder="$t('spaces.conversation.input.placeholder')"
        @keydown.enter.exact="submit"
        @keydown.meta.enter="submit"
        @keydown.ctrl.enter="submit"
      />
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
        <slot name="extensions" />
        <UiIconButton v-if="busy" :label="$t('spaces.conversation.stop')" size="sm" @click="emit('cancel')">
          <CircleStop class="size-3.5" />
        </UiIconButton>
        <UiIconButton
          v-else
          :label="$t('spaces.conversation.input.send')"
          :shortcut="$t('spaces.conversation.input.shortcutHint')"
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
      {{ attachmentError }}
    </p>
  </footer>
</template>

<style scoped>
.composer-input {
  scrollbar-width: none;
}
.composer-input::-webkit-scrollbar {
  display: none;
}
</style>
