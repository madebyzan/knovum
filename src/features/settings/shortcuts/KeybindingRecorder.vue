<script setup lang="ts">
import { AlertTriangle, X } from "@lucide/vue";
import { DialogClose, DialogContent, DialogOverlay, DialogPortal, DialogRoot, DialogTitle } from "reka-ui";
import { computed, onMounted, onUnmounted, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { parseKeyboardEventToChord } from "../../../foundation/shortcuts/shortcut-chord";
import { findShortcutConflict } from "../../../foundation/shortcuts/shortcut-conflicts";
import { useShortcutsStore } from "../../../foundation/shortcuts/shortcut-store";
import type { AppCommand } from "../../../shared/contracts/app-command";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiKbd from "../../../shared/ui/UiKbd.vue";

const props = defineProps<{
  title: string;
  targetCommandId: string;
  currentChord?: string | undefined;
  commands: readonly AppCommand[];
}>();

const emit = defineEmits<{
  (e: "save", chord: string): void;
}>();

const open = defineModel<boolean>("open", { required: true });
const { t } = useI18n();
const shortcutsStore = useShortcutsStore();

const recordedChord = shallowRef<string | null>(null);

watch(open, (isOpen) => {
  if (isOpen) {
    recordedChord.value = props.currentChord ?? null;
  }
});

const conflictCommand = computed(() => {
  if (!recordedChord.value) return undefined;
  return findShortcutConflict(
    props.targetCommandId,
    recordedChord.value,
    props.commands,
    shortcutsStore.customShortcuts,
  );
});

function handleKeyDown(event: KeyboardEvent) {
  if (!open.value) return;
  event.preventDefault();
  event.stopPropagation();

  if (event.key === "Escape" && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    open.value = false;
    return;
  }

  const chord = parseKeyboardEventToChord(event);
  if (chord) {
    recordedChord.value = chord;
  }
}

onMounted(() => {
  globalThis.addEventListener("keydown", handleKeyDown, { capture: true });
});

onUnmounted(() => {
  globalThis.removeEventListener("keydown", handleKeyDown, { capture: true });
});

function save() {
  if (recordedChord.value) {
    emit("save", recordedChord.value);
    open.value = false;
  }
}
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-50 bg-surface-scrim" />
      <DialogContent
        class="fixed top-1/2 left-1/2 z-50 flex w-100 -translate-x-1/2 -translate-y-1/2 flex-col rounded-xl border border-stroke-default material-overlay p-5 shadow-2xl select-none"
      >
        <div class="flex items-center justify-between">
          <DialogTitle class="text-title-sm font-semibold tracking-tight text-fg-primary">
            {{ t("settings.shortcuts.recordTitle") }}: {{ props.title }}
          </DialogTitle>
          <DialogClose as-child>
            <UiIconButton :label="t('common.close')" size="sm">
              <X class="size-3.5" :stroke-width="1.6" />
            </UiIconButton>
          </DialogClose>
        </div>

        <div
          class="mt-4 flex min-h-24 flex-col items-center justify-center rounded-lg border border-dashed border-stroke-default bg-surface-muted/50 p-4"
        >
          <UiKbd v-if="recordedChord" :shortcut="recordedChord" variant="badge" class="scale-125" />
          <div v-else class="text-caption text-fg-tertiary">
            {{ t("settings.shortcuts.pressKeys") }}
          </div>
        </div>

        <div
          v-if="conflictCommand"
          class="mt-3 flex items-center gap-2 rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-caption text-amber-500"
        >
          <AlertTriangle class="size-4 shrink-0" />
          <span>{{ t("settings.shortcuts.inAppConflict", { command: conflictCommand.title }) }}</span>
        </div>

        <div class="mt-5 flex justify-end gap-2">
          <UiButton variant="ghost" size="sm" @click="open = false">
            {{ t("common.cancel") }}
          </UiButton>
          <UiButton variant="primary" size="sm" :disabled="!recordedChord" @click="save">
            {{ t("common.save") }}
          </UiButton>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
