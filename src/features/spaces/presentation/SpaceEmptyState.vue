<script setup lang="ts">
import { FilePlus2, Inbox, StickyNote } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../shared/ui/UiButton.vue";
import { importSpaceFromPicker } from "../assets/import-flow";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import { useSpaceStore } from "../session/space-store";

const store = useSpaceStore();
const { t } = useI18n();

function addNote() {
  void store.addContent(createSpaceNotePayload(t("spaces.editor.newNote")));
}

async function importFile() {
  try {
    await importSpaceFromPicker(store, {
      title: t("spaces.transfer.importingResource"),
      preparing: t("spaces.transfer.importPreparing"),
    });
  } catch (error) {
    toast.error(t("spaces.transfer.importFailed"), { description: String(error) });
  }
}
</script>

<template>
  <div class="pointer-events-none absolute inset-0 grid place-items-center">
    <div class="pointer-events-auto -translate-y-6 text-center">
      <Inbox class="mx-auto size-(--knovum-icon-size-lg) text-foreground-tertiary" :stroke-width="1.5" />
      <p class="mt-3 text-body-lg font-medium">{{ t("spaces.space.emptyTitle") }}</p>
      <p class="mt-1 text-caption text-foreground-tertiary">{{ t("spaces.space.emptyDescription") }}</p>
      <div class="mt-4 flex justify-center gap-2">
        <UiButton size="sm" variant="secondary" @click="addNote">
          <StickyNote class="mr-1.5 size-(--knovum-icon-size-xs)" />{{ t("spaces.editor.newNote") }}
        </UiButton>
        <UiButton size="sm" variant="secondary" @click="importFile">
          <FilePlus2 class="mr-1.5 size-(--knovum-icon-size-xs)" />{{ t("spaces.transfer.import") }}
        </UiButton>
      </div>
    </div>
  </div>
</template>
