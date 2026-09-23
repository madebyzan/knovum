<script setup lang="ts">
import { computed } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSlider from "../../../../shared/ui/UiSlider.vue";
import type { VideoEditProject } from "../../video-editing/project";

const project = defineModel<VideoEditProject>({ required: true });
const props = defineProps<{
  selectedClipId?: string | undefined;
  selectedTextId?: string | undefined;
}>();
const selectedClip = computed(() =>
  project.value.videoTrack.clips.find(({ id }) => id === props.selectedClipId),
);
const selectedText = computed(() =>
  project.value.textTrack.items.find(({ id }) => id === props.selectedTextId),
);

function patchClip(patch: Partial<NonNullable<typeof selectedClip.value>>) {
  project.value = {
    ...project.value,
    videoTrack: {
      clips: project.value.videoTrack.clips.map((clip) =>
        clip.id === props.selectedClipId ? { ...clip, ...patch } : clip,
      ),
    },
  };
}

function patchText(patch: Partial<NonNullable<typeof selectedText.value>>) {
  project.value = {
    ...project.value,
    textTrack: {
      items: project.value.textTrack.items.map((item) =>
        item.id === props.selectedTextId ? { ...item, ...patch } : item,
      ),
    },
  };
}

function rotate() {
  const rotation = ((project.value.transform.rotation + 90) % 360) as 0 | 90 | 180 | 270;
  project.value = { ...project.value, transform: { ...project.value.transform, rotation } };
}
</script>

<template>
  <aside class="w-64 shrink-0 overflow-y-auto border-l border-border bg-panel p-3">
    <h2 class="text-label font-semibold">{{ $t("spaces.media.videoSettings") }}</h2>
    <div class="mt-2 grid grid-cols-2 gap-1">
      <UiButton size="sm" @click="rotate"
        >{{ $t("spaces.media.rotate") }} {{ project.transform.rotation }}°</UiButton
      >
      <UiButton
        size="sm"
        @click="
          project = {
            ...project,
            transform: { ...project.transform, flipHorizontal: !project.transform.flipHorizontal },
          }
        "
        >{{ $t("spaces.media.flipHorizontal") }}</UiButton
      >
    </div>
    <template v-if="selectedClip">
      <h3 class="mt-5 text-caption font-semibold">{{ $t("spaces.media.selectedClip") }}</h3>
      <label class="mt-2 block text-caption text-foreground-secondary">{{ $t("spaces.media.volume") }}</label>
      <UiSlider
        :model-value="selectedClip.volume"
        :label="$t('spaces.media.volume')"
        :min="0"
        :max="2"
        :step="0.05"
        @update:model-value="patchClip({ volume: $event })"
      />
    </template>
    <template v-if="selectedText">
      <h3 class="mt-5 text-caption font-semibold">{{ $t("spaces.media.selectedText") }}</h3>
      <UiInput
        class="mt-2"
        size="sm"
        :model-value="selectedText.text"
        @update:model-value="patchText({ text: $event })"
      />
      <div class="mt-2 grid grid-cols-3 gap-1">
        <UiButton
          v-for="placement in ['top', 'center', 'bottom'] as const"
          :key="placement"
          size="sm"
          :variant="selectedText.placement === placement ? 'primary' : 'secondary'"
          @click="patchText({ placement })"
          >{{ $t(`spaces.media.${placement}`) }}</UiButton
        >
      </div>
    </template>
  </aside>
</template>
