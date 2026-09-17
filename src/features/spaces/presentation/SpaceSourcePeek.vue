<script setup lang="ts">
import { ArrowUpRight, FilePlus2, LocateFixed } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import { knowledgeLocationLabel } from "../knowledge/model/location";
import { useSourcePeek } from "../knowledge/sources/use-source-peek";
import SpaceTextQuote from "./SpaceTextQuote.vue";

const { t } = useI18n();
const {
  store,
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
  <section v-if="sourcePeek" class="flex h-full min-h-0 flex-col bg-surface">
    <div class="min-h-0 flex-1 overflow-y-auto p-3">
      <div v-if="sourceContext.length" class="space-y-2">
        <div
          v-for="chunk in sourceContext"
          :key="chunk.id"
          class="border-b border-divider py-2.5 last:border-b-0"
        >
          <p class="mb-1 text-micro text-foreground-tertiary">{{ knowledgeLocationLabel(chunk.locator) }}</p>
          <p v-if="chunk.metadata?.role === 'document_ocr'" class="mb-1 text-micro text-foreground-secondary">
            {{ t("spaces.processing.documentOcrEvidence") }}
          </p>
          <p
            v-if="chunk.metadata?.role === 'document_interpretation'"
            class="mb-1 text-micro text-foreground-secondary"
          >
            {{ t("spaces.processing.documentInterpretationEvidence") }}
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
      <p v-else class="text-caption text-foreground-tertiary">
        {{ t("spaces.assistant.sourcePeekNoText") }}
      </p>
    </div>

    <footer class="flex flex-wrap items-center justify-end gap-1 border-t border-window-hairline px-2.5 py-2">
      <UiButton size="sm" variant="ghost" @click="extractSource">
        <FilePlus2 class="size-(--knovum-icon-size-xs)" />{{ t("spaces.assistant.extractAsEvidence") }}
      </UiButton>
      <UiButton size="sm" variant="ghost" @click="revealSource">
        <LocateFixed class="size-(--knovum-icon-size-xs)" />{{ t("spaces.canvas.revealInCanvas") }}
      </UiButton>
      <UiButton v-if="editableSource" size="sm" @click="store.openEditorAtSource(sourcePeek)">
        {{ t("spaces.transfer.openInEditor") }}
      </UiButton>
      <template v-if="hasAsset">
        <UiButton v-if="preciseExternal" size="sm" variant="secondary" @click="openSourceAtLocation">
          {{ t("spaces.assistant.openAtSource") }}
        </UiButton>
        <UiButton size="sm" @click="openSourceExternally">
          <ArrowUpRight class="size-(--knovum-icon-size-xs)" />{{ t("spaces.transfer.openExternally") }}
        </UiButton>
      </template>
    </footer>
  </section>
</template>
