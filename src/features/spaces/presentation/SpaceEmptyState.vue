<script setup lang="ts">
import { FilePlus2, Inbox, StickyNote } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { errorMessage } from "../../../foundation/errors/error-format";
import UiButton from "../../../shared/ui/UiButton.vue";
import { importSpaceFromPicker } from "../assets/import-flow";
import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import { useSpaceStore } from "../session/space-store";

const store = useSpaceStore();
const operations = useNodeOperationGate();
const { t } = useI18n();

function addNote() {
  if (!operations.allows("create")) return;
  void store.addContent(createSpaceNotePayload(t("spaces.editor.note.new")));
}

async function importFile() {
  if (!operations.allows("create")) return;
  try {
    await importSpaceFromPicker(store, {
      title: t("spaces.importExport.progress.resource"),
      preparing: t("spaces.importExport.progress.preparing"),
    });
  } catch (error) {
    toast.error(t("spaces.importExport.importFailed"), { description: errorMessage(error) });
  }
}
</script>

<template>
  <div class="pointer-events-none absolute inset-0 grid place-items-center">
    <div class="pointer-events-auto -translate-y-6 text-center">
      <Inbox class="mx-auto size-5 text-fg-tertiary" :stroke-width="1.5" />
      <p class="mt-3 text-title-sm font-medium">{{ t("spaces.space.empty.title") }}</p>
      <p class="mt-1 text-caption text-fg-tertiary">{{ t("spaces.space.empty.description") }}</p>
      <div class="mt-4 flex justify-center gap-2">
        <UiButton size="sm" variant="secondary" :disabled="!operations.allows('create')" @click="addNote">
          <StickyNote class="mr-1.5 size-3" />{{ t("spaces.editor.note.new") }}
        </UiButton>
        <UiButton size="sm" variant="secondary" :disabled="!operations.allows('create')" @click="importFile">
          <FilePlus2 class="mr-1.5 size-3" />{{ t("spaces.importExport.import") }}
        </UiButton>
      </div>
    </div>
  </div>
</template>
