<script setup lang="ts">
import { Blend, Film, Type } from "@lucide/vue";
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from "reka-ui";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import SpaceVideoMaterialBrowser from "./SpaceVideoMaterialBrowser.vue";
import SpaceVideoTextTrackPicker from "./SpaceVideoTextTrackPicker.vue";
import SpaceVideoTransitionEditor from "./SpaceVideoTransitionEditor.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const tab = shallowRef<"media" | "text" | "transitions">("media");
const tabs = computed(() => [
  { value: "media" as const, icon: Film, label: t("spaces.media.videoEditor.library.media") },
  { value: "text" as const, icon: Type, label: t("spaces.media.videoEditor.library.text") },
  {
    value: "transitions" as const,
    icon: Blend,
    label: t("spaces.media.videoEditor.library.transitions"),
  },
]);
function setTab(value: unknown) {
  if (value === "media" || value === "text" || value === "transitions") tab.value = value;
}
</script>

<template>
  <TabsRoot
    :model-value="tab"
    class="flex h-full min-h-0 flex-col bg-surface-panel"
    @update:model-value="setTab"
  >
    <TabsList
      class="grid shrink-0 grid-cols-3 gap-1 border-b border-stroke-divider bg-surface-panel p-1.5"
      :aria-label="t('spaces.media.videoEditor.library.title')"
    >
      <TabsTrigger
        v-for="entry in tabs"
        :key="entry.value"
        :value="entry.value"
        class="flex items-center justify-center gap-1.5 rounded-md px-2 py-1 text-caption font-medium whitespace-nowrap text-fg-secondary transition-colors outline-none hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus data-[state=active]:bg-surface-selected data-[state=active]:text-fg-accent data-[state=active]:shadow-xs"
      >
        <component :is="entry.icon" class="size-3.5 shrink-0" />
        <span class="truncate">{{ entry.label }}</span>
      </TabsTrigger>
    </TabsList>
    <TabsContent value="media" class="flex min-h-0 flex-1 flex-col outline-none">
      <SpaceVideoMaterialBrowser :editor="props.editor" />
    </TabsContent>
    <TabsContent value="text" class="flex min-h-0 flex-1 flex-col outline-none">
      <SpaceVideoTextTrackPicker :editor="props.editor" />
    </TabsContent>
    <TabsContent value="transitions" class="flex min-h-0 flex-1 flex-col outline-none">
      <SpaceVideoTransitionEditor :editor="props.editor" />
    </TabsContent>
  </TabsRoot>
</template>
