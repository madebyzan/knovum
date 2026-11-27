<script setup lang="ts">
import { BookOpen, ChevronRight, FileText, Upload } from "@lucide/vue";
import { computed, inject, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { workspaceRightSidebarKey } from "../../../../shared/contracts/workspace-right-sidebar";
import UiKbd from "../../../../shared/ui/UiKbd.vue";
import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import { usePickerImport } from "../../assets/use-picker-import";
import { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import { getCanvasEditorSession } from "../../canvas/editor-session";
import { useCanvasTextCreation } from "../../canvas/presentation/use-canvas-text-creation";
import { createContextDock } from "../../canvas/use-context-dock";
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
const rightSidebar = inject(workspaceRightSidebarKey, undefined);
const dock = rightSidebar ? createContextDock(rightSidebar) : undefined;
const loadingSample = shallowRef(false);

const SHORTCUT_HINTS = [
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

async function handleLoadSample() {
  if (loadingSample.value || !nodeOperations.allows("create")) return;
  loadingSample.value = true;
  try {
    await store.importSampleSpace();
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    loadingSample.value = false;
  }
}

const starterActions = computed(() => [
  {
    id: "import",
    icon: Upload,
    title: t("spaces.space.empty.actions.importTitle"),
    description: t("spaces.space.empty.actions.importDescription"),
    shortcut: undefined,
    action: (): void => void handleImport(),
  },
  ...(dock
    ? [
        {
          id: "ask",
          icon: UiKnovumAiIcon,
          title: t("spaces.space.empty.actions.askTitle"),
          description: t("spaces.space.empty.actions.askDescription"),
          shortcut: undefined,
          action: (): void => dock.openAsk(),
        },
      ]
    : []),
  {
    id: "note",
    icon: FileText,
    title: t("spaces.space.empty.actions.noteTitle"),
    description: t("spaces.space.empty.actions.noteDescription"),
    shortcut: "N",
    action: (): void => void createNote(),
  },
]);
</script>

<template>
  <div class="pointer-events-none absolute inset-0 grid place-items-center p-6 select-none">
    <div
      class="flex w-full max-w-[680px] -translate-y-3 flex-col items-center gap-6 md:flex-row md:items-center md:gap-9"
    >
      <img
        :src="elfImage"
        alt=""
        class="w-48 shrink-0 object-contain select-none md:w-56"
        draggable="false"
      />

      <div class="flex w-full max-w-[400px] flex-col text-left">
        <h2
          class="text-title font-semibold tracking-title text-fg-primary"
          :aria-label="t('spaces.space.empty.title')"
        >
          {{ t("spaces.space.empty.titlePrefix")
          }}<span class="relative inline-block text-fg-primary">
            {{ t("spaces.space.empty.titleHighlight") }}
            <svg
              class="pointer-events-none absolute -bottom-1 -left-[12%] -z-1 h-3 w-[142%] overflow-visible text-fg-accent"
              viewBox="0 0 254 26"
              fill="none"
              aria-hidden="true"
            >
              <path
                d="M 2 22 C 25 18, 70 5, 120 2 C 160 1, 210 2, 252 9 C 254 10, 254 11, 251 12 C 210 13, 160 13, 120 13 C 70 15, 25 24, 2 25 C 0 25, 0 23, 2 22 Z"
                fill="currentColor"
                fill-opacity="0.26"
              />
            </svg>
          </span>
        </h2>

        <p class="mt-1.5 text-caption leading-relaxed text-fg-tertiary">
          {{ t("spaces.space.empty.description") }}
        </p>

        <div class="pointer-events-auto -mx-2.5 mt-3.5 flex flex-col gap-0.5">
          <button
            v-for="item in starterActions"
            :key="item.id"
            type="button"
            class="group flex w-full cursor-default items-center gap-3 rounded-lg px-2.5 py-2 text-left transition-colors duration-100 hover:bg-surface-control-hover focus-visible:ring-1 focus-visible:ring-stroke-focus focus-visible:outline-none active:bg-surface-control-pressed"
            @click="item.action"
          >
            <component
              :is="item.icon"
              class="size-4 shrink-0 text-fg-secondary transition-colors duration-100 group-hover:text-fg-primary"
              :stroke-width="1.6"
            />
            <span class="min-w-0 flex-1">
              <span class="block text-label font-medium text-fg-primary">{{ item.title }}</span>
              <span class="block truncate text-caption text-fg-tertiary">{{ item.description }}</span>
            </span>
            <UiKbd v-if="item.shortcut" :shortcut="item.shortcut" variant="badge" class="shrink-0" />
            <ChevronRight
              v-else
              class="size-3.5 shrink-0 text-fg-tertiary/60 transition-colors duration-100 group-hover:text-fg-secondary"
              :stroke-width="1.6"
            />
          </button>
        </div>

        <div
          class="pointer-events-auto mt-3 flex flex-wrap items-center justify-between gap-2 border-t border-stroke-divider/70 pt-2.5 text-caption text-fg-tertiary"
        >
          <div class="-ml-1.5 flex items-center gap-1">
            <button
              v-for="item in SHORTCUT_HINTS"
              :key="item.key"
              type="button"
              class="inline-flex h-6 cursor-default items-center gap-1.5 rounded-md px-1.5 transition-colors duration-100 hover:bg-surface-control-hover hover:text-fg-primary active:bg-surface-control-pressed"
              @click="item.action"
            >
              <UiKbd :shortcut="item.key" variant="badge" />
              <span>{{ t(item.label) }}</span>
            </button>
          </div>
          <button
            type="button"
            class="-mr-1.5 inline-flex h-6 cursor-default items-center gap-1.5 rounded-md px-1.5 text-fg-secondary transition-colors duration-100 hover:bg-surface-control-hover hover:text-fg-primary active:bg-surface-control-pressed disabled:opacity-40"
            :disabled="loadingSample"
            @click="handleLoadSample"
          >
            <BookOpen class="size-3.5 shrink-0 opacity-75" :stroke-width="1.6" />
            <span>{{ t("spaces.space.empty.loadSample") }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
