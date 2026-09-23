<script setup lang="ts">
import { X } from "@lucide/vue";
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from "reka-ui";
import { computed, shallowRef, watch } from "vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import type { VideoEditProject } from "../../video-editing/project";
import SpaceVideoCaptionList from "./SpaceVideoCaptionList.vue";
import SpaceVideoClipInspector from "./SpaceVideoClipInspector.vue";
import SpaceVideoTextInspector from "./SpaceVideoTextInspector.vue";

const props = defineProps<{
  modelValue: VideoEditProject;
  sourceDurationMs: number;
  selectedClipId?: string | undefined;
  selectedTextId?: string | undefined;
}>();
const emit = defineEmits<{
  edit: [project: VideoEditProject, mergeKey?: string];
  close: [];
  selectText: [id: string];
  addText: [];
}>();
const selectedClip = computed(() =>
  props.modelValue.videoTrack.clips.find(({ id }) => id === props.selectedClipId),
);
const selectedText = computed(() =>
  props.modelValue.textTrack.items.find(({ id }) => id === props.selectedTextId),
);
const tab = shallowRef<"captions" | "text">("text");
watch(
  () => !!props.selectedTextId,
  (isText, wasText) => {
    if (isText && !wasText) tab.value = "text";
  },
);
function setTab(value: unknown) {
  if (value === "captions" || value === "text") tab.value = value;
}
function addText() {
  tab.value = "text";
  emit("addText");
}
</script>

<template>
  <aside
    class="absolute inset-y-3 right-3 z-30 flex min-h-0 w-80 flex-col overflow-hidden rounded-section border border-border bg-overlay shadow-popover"
    :aria-label="$t('spaces.media.videoSettings')"
    @keydown.esc.prevent.stop="emit('close')"
  >
    <TabsRoot
      v-if="selectedText"
      :model-value="tab"
      class="flex min-h-0 flex-1 flex-col"
      @update:model-value="setTab"
    >
      <div class="flex h-11 shrink-0 items-center border-b border-divider px-3">
        <TabsList :aria-label="$t('spaces.media.textTools')" class="flex h-full flex-1 items-stretch gap-5">
          <TabsTrigger
            value="captions"
            class="border-b-2 border-transparent px-1 text-caption text-foreground-secondary outline-none focus-visible:ring-2 focus-visible:ring-focus-ring data-[state=active]:border-foreground data-[state=active]:font-semibold data-[state=active]:text-foreground"
            >{{ $t("spaces.media.captionsTab") }}</TabsTrigger
          >
          <TabsTrigger
            value="text"
            class="border-b-2 border-transparent px-1 text-caption text-foreground-secondary outline-none focus-visible:ring-2 focus-visible:ring-focus-ring data-[state=active]:border-foreground data-[state=active]:font-semibold data-[state=active]:text-foreground"
            >{{ $t("spaces.media.textTab") }}</TabsTrigger
          >
        </TabsList>
        <UiIconButton :label="$t('spaces.media.closeInspector')" size="sm" @click="emit('close')">
          <X class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </div>
      <TabsContent value="captions" class="flex min-h-0 flex-1 flex-col outline-none">
        <SpaceVideoCaptionList
          :items="modelValue.textTrack.items"
          :selected-text-id="selectedTextId"
          @select="emit('selectText', $event)"
          @add="addText"
        />
      </TabsContent>
      <TabsContent value="text" class="min-h-0 flex-1 outline-none">
        <UiScrollArea class="h-full">
          <SpaceVideoTextInspector
            :project="modelValue"
            :item="selectedText"
            @edit="(project, mergeKey) => emit('edit', project, mergeKey)"
          />
        </UiScrollArea>
      </TabsContent>
    </TabsRoot>
    <template v-else-if="selectedClip">
      <div class="flex h-11 shrink-0 items-center justify-between border-b border-divider px-3">
        <h2 class="text-caption font-semibold">{{ $t("spaces.media.selectedClip") }}</h2>
        <UiIconButton :label="$t('spaces.media.closeInspector')" size="sm" @click="emit('close')">
          <X class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </div>
      <UiScrollArea class="min-h-0 flex-1">
        <SpaceVideoClipInspector
          :project="modelValue"
          :clip="selectedClip"
          :source-duration-ms="sourceDurationMs"
          @edit="(project, mergeKey) => emit('edit', project, mergeKey)"
        />
      </UiScrollArea>
    </template>
  </aside>
</template>
