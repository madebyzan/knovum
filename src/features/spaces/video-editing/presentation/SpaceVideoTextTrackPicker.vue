<script setup lang="ts">
import { Captions, Download, Plus, Upload } from "@lucide/vue";
import { sortBy } from "es-toolkit";
import { search } from "fast-fuzzy";
import { computed, nextTick, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import { formatVideoTimelineTime } from "../timecode";
import { useVideoCaptions } from "./use-video-captions";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { project, player, selection, actions } = props.editor;
const captions = useVideoCaptions(props.editor);
const query = shallowRef("");
const selectedRow = shallowRef<HTMLElement | null>(null);

const allItems = computed(() => sortBy(project.value?.textTrack.items ?? [], [({ startMs }) => startMs]));
const items = computed(() => {
  const needle = query.value.trim();
  return needle
    ? sortBy(search(needle, allItems.value, { keySelector: ({ text }) => text }), [({ startMs }) => startMs])
    : allItems.value;
});
const exportMenu = computed(() => [
  {
    id: "srt",
    label: t("spaces.media.videoEditor.captions.exportSrt"),
    onSelect: () => void captions.exportSubtitles("srt"),
  },
  {
    id: "vtt",
    label: t("spaces.media.videoEditor.captions.exportVtt"),
    onSelect: () => void captions.exportSubtitles("vtt"),
  },
]);

function isOnScreen(startMs: number, endMs: number) {
  const now = player.displayMs.value;
  return now >= startMs && now < endMs;
}

watch(
  () => selection.text.value?.id,
  async () => {
    await nextTick();
    selectedRow.value?.scrollIntoView({ block: "nearest" });
  },
);
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <div class="shrink-0 space-y-1.5 border-b border-stroke-divider/70 px-2 py-2">
      <UiSearchInput
        v-model="query"
        size="sm"
        :label="t('spaces.media.videoEditor.captions.search')"
        :placeholder="t('spaces.media.videoEditor.captions.search')"
      />
      <UiButton
        v-if="captions.hasTranscript.value"
        size="sm"
        variant="secondary"
        class="w-full justify-center"
        @click="captions.generateFromTranscript()"
      >
        <Captions class="size-3.5" />{{ t("spaces.media.videoEditor.captions.fromTranscript") }}
      </UiButton>
      <div class="flex items-center gap-1">
        <UiButton size="sm" class="min-w-0 flex-1 justify-center" @click="actions.addText()">
          <Plus class="size-3.5" />{{ t("spaces.media.videoEditor.text.add") }}
        </UiButton>
        <UiIconButton
          size="sm"
          tooltip
          :label="t('spaces.media.videoEditor.captions.importFile')"
          @click="captions.importSubtitles()"
        >
          <Upload class="size-3.5" />
        </UiIconButton>
        <UiDropdownMenu :items="exportMenu" align="end">
          <template #trigger>
            <UiIconButton
              size="sm"
              tooltip
              :disabled="!allItems.length"
              :label="t('spaces.media.videoEditor.captions.exportMenu')"
            >
              <Download class="size-3.5" />
            </UiIconButton>
          </template>
        </UiDropdownMenu>
      </div>
    </div>
    <UiScrollArea class="min-h-0 flex-1">
      <div class="space-y-1 p-2">
        <button
          v-for="(item, index) in items"
          :key="item.id"
          :ref="
            (value) => item.id === selection.text.value?.id && (selectedRow = value as HTMLElement | null)
          "
          type="button"
          class="relative w-full rounded-md px-2.5 py-2 text-left transition-colors focus-visible:outline-2 focus-visible:outline-stroke-focus"
          :class="
            item.id === selection.text.value?.id
              ? 'bg-surface-selected text-fg-accent'
              : isOnScreen(item.startMs, item.endMs)
                ? 'bg-surface-control text-fg-primary'
                : 'text-fg-primary hover:bg-surface-control-hover'
          "
          :aria-pressed="item.id === selection.text.value?.id"
          @click="selection.select({ kind: 'text', id: item.id })"
        >
          <span
            v-if="isOnScreen(item.startMs, item.endMs)"
            class="absolute top-2 bottom-2 left-0.5 w-0.5 rounded-full bg-surface-accent"
            aria-hidden="true"
          />
          <span
            class="flex items-center justify-between gap-2 font-mono text-caption-sm text-fg-tertiary tabular-nums"
          >
            <span
              >#{{ index + 1 }} · {{ formatVideoTimelineTime(item.startMs) }} –
              {{ formatVideoTimelineTime(item.endMs) }}</span
            >
            <span>{{ ((item.endMs - item.startMs) / 1000).toFixed(1) }}s</span>
          </span>
          <span class="mt-0.5 block truncate text-caption">{{ item.text }}</span>
        </button>
        <p v-if="!items.length" class="px-2 py-6 text-center text-caption text-fg-secondary">
          {{
            query
              ? t("spaces.media.videoEditor.captions.noMatches")
              : t("spaces.media.videoEditor.captions.empty")
          }}
        </p>
      </div>
    </UiScrollArea>
  </div>
</template>
