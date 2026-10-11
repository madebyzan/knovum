<script setup lang="ts">
import { ArrowUpRight, FilePlus2, LocateFixed } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import UiButton from "../../../../shared/ui/UiButton.vue";
import { useSpaceStore } from "../../session/space-store";
import { knowledgeLocationLabel } from "./knowledge-location-label";
import SpaceTextQuote from "./SpaceTextQuote.vue";
import { useSourcePeek } from "./use-source-peek";

const { t } = useI18n();
const store = useSpaceStore();
async function openEditor() {
  const target = store.sourcePeek;
  if (!target) return;
  try {
    await store.openEditorAtSource(target);
  } catch (error) {
    if (!isApplicationLeaveCancelled(error)) toast.error(getErrorMessage(error));
  }
}
const {
  sourcePeek,
  sourceContext,
  preciseExternal,
  editableSource,
  hasAsset,
  revealSource,
  extractSource,
  openSourceExternally,
  openSourceAtLocation,
} = useSourcePeek();
</script>

<template>
  <section v-if="sourcePeek" class="flex h-full min-h-0 flex-col bg-surface-muted">
    <div class="min-h-0 flex-1 overflow-y-auto p-3">
      <div v-if="sourceContext.length" class="space-y-2">
        <div
          v-for="chunk in sourceContext"
          :key="chunk.id"
          class="border-b border-stroke-divider py-2.5 last:border-b-0"
        >
          <p class="mb-1 text-caption-sm text-fg-tertiary">
            {{ knowledgeLocationLabel(chunk.locator) }}
          </p>
          <p v-if="chunk.metadata?.role === 'document_ocr'" class="mb-1 text-caption-sm text-fg-secondary">
            {{ t("spaces.processing.document.ocrEvidence") }}
          </p>
          <p
            v-if="chunk.metadata?.role === 'document_interpretation'"
            class="mb-1 text-caption-sm text-fg-secondary"
          >
            {{ t("spaces.processing.document.interpretationEvidence") }}
          </p>
          <SpaceTextQuote
            :text="chunk.text"
            v-bind="
              chunk.id === sourcePeek.chunkId && sourcePeek.locator.quote?.exact
                ? { exact: sourcePeek.locator.quote.exact }
                : {}
            "
          />
        </div>
      </div>
      <SpaceTextQuote
        v-else-if="sourcePeek.text"
        :text="sourcePeek.text"
        v-bind="sourcePeek.locator.quote?.exact ? { exact: sourcePeek.locator.quote.exact } : {}"
      />
      <p v-else class="text-caption text-fg-tertiary">
        {{ t("spaces.conversation.sourcePeek.noText") }}
      </p>
    </div>

    <footer
      class="flex flex-wrap items-center justify-end gap-1 border-t border-(--knovum-window-border-color) px-2.5 py-2"
    >
      <UiButton size="sm" variant="ghost" @click="extractSource">
        <FilePlus2 class="size-3" />{{ t("spaces.conversation.evidence.extract") }}
      </UiButton>
      <UiButton size="sm" variant="ghost" @click="revealSource">
        <LocateFixed class="size-3" />{{ t("spaces.canvas.revealInCanvas") }}
      </UiButton>
      <UiButton v-if="editableSource" size="sm" @click="openEditor">
        {{ t("spaces.importExport.openInEditor") }}
      </UiButton>
      <template v-if="hasAsset">
        <UiButton v-if="preciseExternal" size="sm" variant="secondary" @click="openSourceAtLocation">
          {{ t("spaces.conversation.sourcePeek.open") }}
        </UiButton>
        <UiButton size="sm" @click="openSourceExternally">
          <ArrowUpRight class="size-3" />{{ t("spaces.importExport.openExternally") }}
        </UiButton>
      </template>
    </footer>
  </section>
</template>
