<script setup lang="ts">
import { ArrowUpRight, Clock, FileEdit, FilePlus2, FileText, LocateFixed, Quote } from "@lucide/vue";
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
  <section v-if="sourcePeek" class="flex h-full min-h-0 flex-col bg-surface-panel">
    <div class="min-h-0 flex-1 space-y-3.5 overflow-y-auto p-4">
      <div class="space-y-2 rounded-xl border border-stroke-divider bg-surface-control/30 p-3.5">
        <div class="flex items-start gap-2.5">
          <div
            class="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-lg bg-surface-control text-fg-secondary"
          >
            <FileText class="size-4" :stroke-width="1.6" />
          </div>
          <div class="min-w-0 flex-1">
            <h3
              class="text-body leading-snug font-semibold break-words text-fg-primary"
              :title="sourcePeek.nodeTitle"
            >
              {{ sourcePeek.nodeTitle }}
            </h3>
          </div>
        </div>

        <div
          v-if="knowledgeLocationLabel(sourcePeek.locator) || sourcePeek.locator.quote?.exact"
          class="flex flex-wrap items-center gap-1.5 pt-0.5"
        >
          <span
            v-if="knowledgeLocationLabel(sourcePeek.locator)"
            class="inline-flex items-center gap-1 rounded-md border border-stroke-divider/60 bg-surface-raised px-2 py-0.5 text-caption-sm font-medium text-fg-secondary shadow-2xs"
          >
            <Clock class="size-3 text-fg-tertiary" />{{ knowledgeLocationLabel(sourcePeek.locator) }}
          </span>
          <span
            v-if="sourcePeek.locator.quote?.exact"
            class="inline-flex items-center gap-1 rounded-md bg-surface-accent-subtle px-2 py-0.5 text-caption-sm font-medium text-fg-primary"
          >
            <Quote class="size-3 text-fg-secondary" />{{ t("spaces.inspector.fields.source") }}
          </span>
        </div>
      </div>

      <div class="space-y-3 rounded-xl border border-stroke-divider bg-surface-raised p-4 shadow-2xs">
        <div class="flex items-center gap-1.5 text-caption font-medium text-fg-tertiary">
          <Quote class="size-3.5 text-fg-tertiary" />
          <span>{{ t("spaces.conversation.evidence.sourceRationale") }}</span>
        </div>

        <div v-if="sourceContext.length" class="space-y-3">
          <div
            v-for="chunk in sourceContext"
            :key="chunk.id"
            class="space-y-1.5 border-b border-stroke-divider/50 pb-3 last:border-b-0 last:pb-0"
          >
            <p class="text-caption-sm text-fg-tertiary">{{ knowledgeLocationLabel(chunk.locator) }}</p>
            <SpaceTextQuote
              :text="chunk.text"
              :exact="chunk.id === sourcePeek.chunkId ? sourcePeek.locator.quote?.exact : undefined"
            />
          </div>
        </div>

        <SpaceTextQuote
          v-else-if="sourcePeek.text"
          :text="sourcePeek.text"
          :exact="sourcePeek.locator.quote?.exact"
        />

        <p v-else class="py-3 text-center text-caption text-fg-tertiary">
          {{ t("spaces.conversation.sourcePeek.noText") }}
        </p>
      </div>
    </div>

    <footer class="shrink-0 space-y-2 border-t border-(--knovum-window-border-color) bg-surface-panel p-3">
      <UiButton
        variant="primary"
        size="md"
        class="w-full justify-center gap-2 shadow-xs"
        @click="extractSource"
      >
        <FilePlus2 class="size-4" />
        <span>{{ t("spaces.conversation.evidence.extract") }}</span>
      </UiButton>
      <div class="flex items-center gap-2">
        <UiButton variant="secondary" size="sm" class="flex-1 justify-center gap-1.5" @click="revealSource">
          <LocateFixed class="size-3.5" />
          <span>{{ t("spaces.canvas.revealInCanvas") }}</span>
        </UiButton>
        <UiButton
          v-if="editableSource"
          variant="secondary"
          size="sm"
          class="flex-1 justify-center gap-1.5"
          @click="openEditor"
        >
          <FileEdit class="size-3.5" />
          <span>{{ t("spaces.importExport.openInEditor") }}</span>
        </UiButton>
        <template v-else-if="hasAsset">
          <UiButton
            v-if="preciseExternal"
            variant="secondary"
            size="sm"
            class="flex-1 justify-center gap-1.5"
            @click="openSourceAtLocation"
          >
            {{ t("spaces.conversation.sourcePeek.open") }}
          </UiButton>
          <UiButton
            variant="secondary"
            size="sm"
            class="flex-1 justify-center gap-1.5"
            @click="openSourceExternally"
          >
            <ArrowUpRight class="size-3.5" />
            <span>{{ t("spaces.importExport.openExternally") }}</span>
          </UiButton>
        </template>
      </div>
    </footer>
  </section>
</template>
