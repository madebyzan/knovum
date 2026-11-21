<script setup lang="ts">
import { Copy, FilePlus2, ListPlus, MessageSquareQuote, StickyNote } from "@lucide/vue";
import { shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { AppendNoteTarget } from "./use-reader-capture";

const props = defineProps<{
  canSave: boolean;
  selectedAppendTarget?: AppendNoteTarget | undefined;
  existingNoteTargets: readonly AppendNoteTarget[];
}>();

const emit = defineEmits<{
  createNote: [];
  createEvidence: [];
  append: [targetId?: string];
  ask: [];
  copy: [];
}>();

const { t } = useI18n();
const pickingNote = shallowRef(false);

watch(
  () => props.selectedAppendTarget,
  () => {
    pickingNote.value = false;
  },
);

function handleAppendTrigger() {
  if (props.selectedAppendTarget) {
    emit("append", props.selectedAppendTarget.id);
    return;
  }
  pickingNote.value = !pickingNote.value;
}

function chooseNote(targetId: string) {
  pickingNote.value = false;
  emit("append", targetId);
}
</script>

<template>
  <div
    role="toolbar"
    :aria-label="t('spaces.conversation.capture.selection')"
    class="flex flex-col rounded-lg border border-stroke-default bg-surface-raised p-1 shadow-popover select-none"
    @mousedown.prevent
  >
    <div class="flex items-center gap-0.5">
      <button
        type="button"
        :aria-label="t('spaces.conversation.capture.saveSelection')"
        :disabled="!canSave"
        class="inline-flex h-7 items-center gap-1.5 rounded-md px-2 text-caption font-medium text-fg-primary transition-colors hover:bg-surface-control-hover disabled:opacity-40"
        @click="emit('createNote')"
      >
        <StickyNote class="size-3.5 text-fg-secondary" />
        <span class="whitespace-nowrap">{{ t("spaces.conversation.capture.saveSelection") }}</span>
      </button>
      <button
        type="button"
        :aria-label="t('spaces.conversation.evidence.extract')"
        :disabled="!canSave"
        class="inline-flex h-7 items-center gap-1.5 rounded-md px-2 text-caption font-medium text-fg-primary transition-colors hover:bg-surface-control-hover disabled:opacity-40"
        @click="emit('createEvidence')"
      >
        <FilePlus2 class="size-3.5 text-fg-secondary" />
        <span class="whitespace-nowrap">{{ t("spaces.conversation.evidence.extract") }}</span>
      </button>
      <span class="mx-0.5 h-4 w-px bg-stroke-divider" aria-hidden="true" />
      <UiIconButton
        v-if="selectedAppendTarget || existingNoteTargets.length"
        :label="
          selectedAppendTarget
            ? t('spaces.conversation.capture.appendTo', { title: selectedAppendTarget.title })
            : t('spaces.conversation.capture.appendTo', { title: '…' })
        "
        tooltip
        size="sm"
        @click="handleAppendTrigger"
      >
        <ListPlus class="size-3.5" />
      </UiIconButton>
      <UiIconButton :label="t('spaces.conversation.capture.followUp')" tooltip size="sm" @click="emit('ask')">
        <MessageSquareQuote class="size-3.5" />
      </UiIconButton>
      <UiIconButton :label="t('spaces.conversation.messages.copy')" tooltip size="sm" @click="emit('copy')">
        <Copy class="size-3.5" />
      </UiIconButton>
    </div>
    <div
      v-if="pickingNote && !selectedAppendTarget && existingNoteTargets.length"
      class="mt-1 max-h-44 min-w-48 overflow-y-auto border-t border-stroke-divider pt-1"
    >
      <button
        v-for="target in existingNoteTargets"
        :key="target.id"
        type="button"
        class="flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-caption text-fg-primary hover:bg-surface-control-hover"
        @click="chooseNote(target.id)"
      >
        <StickyNote class="size-3.5 shrink-0 text-fg-tertiary" />
        <span class="truncate">{{ target.title }}</span>
      </button>
    </div>
  </div>
</template>
