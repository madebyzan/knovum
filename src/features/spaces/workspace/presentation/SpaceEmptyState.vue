<script setup lang="ts">
import { FileText, Keyboard, Mic, Upload } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
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
      class="flex -translate-y-3 flex-col items-center gap-4 md:flex-row md:items-center md:gap-5 lg:gap-7"
    >
      <div class="relative flex shrink-0 items-center justify-center">
        <img
          :src="elfImage"
          alt="Knowledge Mascot"
          class="w-60 object-contain select-none md:w-72 lg:w-80"
          draggable="false"
        />
      </div>

      <div class="flex max-w-md flex-col items-center text-center md:items-start md:text-left">
        <h2
          class="text-2xl font-bold tracking-tight text-fg-primary md:text-[28px]"
          :aria-label="t('spaces.space.empty.title')"
        >
          {{ t("spaces.space.empty.titlePrefix")
          }}<span class="relative inline-block text-fg-primary">
            {{ t("spaces.space.empty.titleHighlight") }}
            <svg
              class="absolute -bottom-1.5 left-0 h-2 w-full overflow-visible text-[#1668DC]"
              viewBox="0 0 56 6"
              fill="none"
              aria-hidden="true"
            >
              <path
                d="M2 4 C18 1.5, 38 4.5, 54 2.2"
                stroke="currentColor"
                stroke-width="2.6"
                stroke-linecap="round"
              />
            </svg>
          </span>
        </h2>

        <p class="mt-2.5 text-caption leading-relaxed text-fg-tertiary">
          {{ t("spaces.space.empty.description") }}
        </p>

        <div
          class="pointer-events-auto mt-6 flex flex-wrap items-center justify-center gap-2.5 md:justify-start"
        >
          <button
            type="button"
            class="inline-flex h-9.5 cursor-pointer items-center gap-2 rounded-lg bg-[#1668DC] px-4 text-sm font-medium text-white shadow-[0_2px_8px_rgba(22,104,220,0.28)] transition-all duration-150 hover:bg-[#155fc9] hover:shadow-[0_4px_12px_rgba(22,104,220,0.36)] active:scale-[0.98] active:bg-[#0e4da8]"
            @click="createNote"
          >
            <FileText class="size-4 shrink-0 text-white/90" :stroke-width="1.8" />
            <span>{{ t("spaces.space.empty.newNote") }}</span>
            <span class="ml-0.5 rounded-sm bg-white/20 px-1.5 py-0.5 text-[11px] font-semibold text-white">
              N
            </span>
          </button>

          <button
            type="button"
            class="inline-flex h-9.5 cursor-pointer items-center gap-2 rounded-lg border border-[#d7dce5] bg-white px-3.5 text-sm font-medium text-fg-primary shadow-[0_1px_2px_rgba(0,0,0,0.05)] transition-all duration-150 hover:border-[#b0b9c9] hover:bg-gray-50/80 hover:shadow-[0_2px_6px_rgba(0,0,0,0.08)] active:scale-[0.98] dark:border-stroke-default dark:bg-surface-raised dark:hover:border-stroke-strong"
            @click="handleImport"
          >
            <Upload class="size-4 shrink-0 text-fg-secondary" :stroke-width="1.8" />
            <span>{{ t("spaces.space.empty.importFiles") }}</span>
          </button>

          <button
            type="button"
            class="inline-flex h-9.5 cursor-pointer items-center gap-2 rounded-lg border border-[#d7dce5] bg-white px-3.5 text-sm font-medium text-fg-primary shadow-[0_1px_2px_rgba(0,0,0,0.05)] transition-all duration-150 hover:border-[#b0b9c9] hover:bg-gray-50/80 hover:shadow-[0_2px_6px_rgba(0,0,0,0.08)] active:scale-[0.98] dark:border-stroke-default dark:bg-surface-raised dark:hover:border-stroke-strong"
            @click="handleRecord"
          >
            <Mic class="size-4 shrink-0 text-fg-secondary" :stroke-width="1.8" />
            <span>{{ t("spaces.space.empty.startRecording") }}</span>
          </button>
        </div>

        <div
          class="pointer-events-auto mt-6 flex flex-wrap items-center justify-center gap-3.5 text-caption text-fg-tertiary md:justify-start"
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
