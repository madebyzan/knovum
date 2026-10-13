<script setup lang="ts">
import { X } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import SpaceVideoAudioInspector from "./SpaceVideoAudioInspector.vue";
import SpaceVideoClipInspector from "./SpaceVideoClipInspector.vue";
import SpaceVideoTextInspector from "./SpaceVideoTextInspector.vue";
import SpaceVideoTransitionInspector from "./SpaceVideoTransitionInspector.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { selection } = props.editor;

const title = computed(() => {
  const kind = selection.selection.value?.kind;
  if (kind === "clip") return t("spaces.media.videoEditor.clip.selected");
  if (kind === "text") return t("spaces.media.videoEditor.tracks.text");
  if (kind === "audio") return t("spaces.media.videoEditor.tracks.audio");
  return t("spaces.media.videoEditor.transitions.title");
});
</script>

<template>
  <aside
    class="flex h-full min-h-0 flex-col overflow-hidden border-l border-stroke-divider bg-surface-panel"
    :aria-label="title"
    @keydown.esc.prevent.stop="selection.clear()"
  >
    <div class="flex h-10 shrink-0 items-center justify-between border-b border-stroke-divider px-3">
      <h2 class="text-caption font-semibold">{{ title }}</h2>
      <UiIconButton
        :label="t('spaces.media.videoEditor.closeInspector')"
        size="sm"
        @click="selection.clear()"
      >
        <X class="size-3.5" />
      </UiIconButton>
    </div>
    <UiScrollArea class="min-h-0 flex-1">
      <SpaceVideoClipInspector v-if="selection.clip.value" :editor="editor" :clip="selection.clip.value" />
      <SpaceVideoTextInspector
        v-else-if="selection.text.value"
        :editor="editor"
        :item="selection.text.value"
      />
      <SpaceVideoAudioInspector
        v-else-if="selection.audio.value"
        :editor="editor"
        :clip="selection.audio.value"
      />
      <SpaceVideoTransitionInspector
        v-else-if="selection.transition.value"
        :editor="editor"
        :transition="selection.transition.value"
      />
    </UiScrollArea>
  </aside>
</template>
