<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { sortBy } from "es-toolkit";
import { search } from "fast-fuzzy";
import { computed, nextTick, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import { formatVideoTimelineTime } from "../timecode";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { project, selection, actions } = props.editor;
const query = shallowRef("");
const selectedRow = shallowRef<HTMLElement | null>(null);

const items = computed(() => {
  const all = project.value?.textTrack.items ?? [];
  const needle = query.value.trim();
  const matched = needle ? search(needle, all, { keySelector: ({ text }) => text }) : all;
  return sortBy(matched, [({ startMs }) => startMs]);
});

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
    <div class="shrink-0 space-y-2 px-2 py-2">
      <UiSearchInput
        v-model="query"
        size="sm"
        :label="t('spaces.media.videoEditor.captions.search')"
        :placeholder="t('spaces.media.videoEditor.captions.search')"
      />
      <UiButton size="sm" class="w-full justify-center" @click="actions.addText()">
        <Plus class="size-3.5" />{{ t("spaces.media.videoEditor.text.add") }}
      </UiButton>
    </div>
    <UiScrollArea class="min-h-0 flex-1">
      <div class="space-y-1 p-2">
        <button
          v-for="item in items"
          :key="item.id"
          :ref="
            (value) => {
              if (item.id === selection.text.value?.id) selectedRow = value as HTMLElement | null;
            }
          "
          type="button"
          class="w-full rounded-md px-2.5 py-2 text-left focus-visible:outline-2 focus-visible:outline-stroke-focus"
          :class="
            item.id === selection.text.value?.id
              ? 'bg-surface-selected text-fg-accent'
              : 'text-fg-primary hover:bg-surface-control-hover'
          "
          :aria-pressed="item.id === selection.text.value?.id"
          @click="selection.select({ kind: 'text', id: item.id })"
        >
          <span class="block text-caption-sm text-fg-tertiary tabular-nums">
            {{ formatVideoTimelineTime(item.startMs) }} – {{ formatVideoTimelineTime(item.endMs) }}
          </span>
          <span class="mt-0.5 block truncate text-caption">{{ item.text }}</span>
        </button>
        <p v-if="!items.length" class="px-2 py-6 text-center text-caption text-fg-secondary">
          {{
            t(
              query
                ? "spaces.media.videoEditor.captions.noMatches"
                : "spaces.media.videoEditor.captions.none",
            )
          }}
        </p>
      </div>
    </UiScrollArea>
  </div>
</template>
