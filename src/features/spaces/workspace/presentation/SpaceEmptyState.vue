<script setup lang="ts">
import { FileText, Keyboard, Mic, Upload } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiKbd from "../../../../shared/ui/UiKbd.vue";
import { usePickerImport } from "../../assets/use-picker-import";
import { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import { getCanvasEditorSession } from "../../canvas/editor-session";
import { useCanvasTextCreation } from "../../canvas/presentation/use-canvas-text-creation";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { useSpaceStore } from "../../session/space-store";
import elfImage from "./assets/canvas-knowledge-elf.png";

const { t } = useI18n();
const store = useSpaceStore();
const tools = useCanvasToolStore();
const nodeOperations = useNodeOperationGate();
const editorSession = getCanvasEditorSession();
const createText = useCanvasTextCreation();
const importFromPicker = usePickerImport();

const SHORTCUT_HINTS = [
  { key: "N", label: "spaces.editor.note.new", action: () => createNote() },
  { key: "T", label: "spaces.editor.text.new", action: () => tools.selectTool("text") },
  { key: "S", label: "spaces.canvas.tools.section", action: () => tools.selectTool("section") },
] as const;

async function createNote() {
  if (!nodeOperations.allows("create")) return;
  const editor = editorSession.activeEditor.value;
  const point = editor ? editor.visibleCenter() : store.nextInsertionPoint();
  await createText("note", point);
}

async function handleImport() {
  if (!nodeOperations.allows("create")) return;
  await importFromPicker();
}

async function handleRecord() {
  if (!nodeOperations.allows("create")) return;
  try {
    await store.beginAudioRecording();
  } catch (error) {
    toast.error(t("spaces.recording.startFailed"), { description: getErrorMessage(error) });
  }
}
</script>

<template>
  <div class="pointer-events-none absolute inset-0 grid place-items-center p-6 select-none">
    <div
      class="flex -translate-y-3 flex-col items-center gap-7 md:flex-row md:items-center md:gap-10 lg:gap-14"
    >
      <div class="relative flex shrink-0 items-center justify-center">
        <img
          :src="elfImage"
          alt="Knowledge Mascot"
          class="w-64 object-contain select-none md:w-76 lg:w-84"
          draggable="false"
        />
      </div>

      <div class="flex max-w-md flex-col items-center text-center md:items-start md:text-left">
        <div class="relative inline-block">
          <h2 class="text-title-md font-medium tracking-(--tracking-title) text-fg-primary">
            {{ t("spaces.space.empty.title") }}
          </h2>
          <svg
            class="absolute right-0 -bottom-2 h-2 w-28 overflow-visible text-content-blue"
            viewBox="0 0 110 8"
            fill="none"
            aria-hidden="true"
          >
            <path
              d="M2 5.5 C30 2, 75 7, 108 3"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
            />
          </svg>
        </div>

        <p class="mt-2.5 text-caption leading-relaxed text-fg-tertiary">
          {{ t("spaces.space.empty.description") }}
        </p>

        <div
          class="pointer-events-auto mt-5 flex flex-wrap items-center justify-center gap-2 md:justify-start"
        >
          <UiButton variant="primary" size="md" @click="createNote">
            <FileText class="size-3.5" :stroke-width="1.6" />
            <span>{{ t("spaces.space.empty.newNote") }}</span>
            <UiKbd shortcut="N" class="ml-1 opacity-70" />
          </UiButton>

          <UiButton variant="secondary" size="md" @click="handleImport">
            <Upload class="size-3.5" :stroke-width="1.6" />
            <span>{{ t("spaces.space.empty.importFiles") }}</span>
          </UiButton>

          <UiButton variant="secondary" size="md" @click="handleRecord">
            <Mic class="size-3.5" :stroke-width="1.6" />
            <span>{{ t("spaces.space.empty.startRecording") }}</span>
          </UiButton>
        </div>

        <div
          class="pointer-events-auto mt-4.5 flex flex-wrap items-center justify-center gap-3 text-caption text-fg-tertiary md:justify-start"
        >
          <span class="inline-flex items-center gap-1.5 text-caption">
            <Keyboard class="size-3.5 shrink-0 opacity-70" :stroke-width="1.5" />
            <span>{{ t("spaces.space.empty.keyboardHint") }}</span>
          </span>
          <button
            v-for="item in SHORTCUT_HINTS"
            :key="item.key"
            type="button"
            class="inline-flex cursor-pointer items-center gap-1.5 transition-colors hover:text-fg-primary"
            @click="item.action"
          >
            <UiKbd :shortcut="item.key" variant="badge" />
            <span>{{ t(item.label) }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
