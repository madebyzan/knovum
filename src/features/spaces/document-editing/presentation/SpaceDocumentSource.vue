<script setup lang="ts">
import { SplitterGroup, SplitterPanel } from "reka-ui";
import { computed, shallowRef } from "vue";

import UiSplitterHandle from "../../../../shared/ui/UiSplitterHandle.vue";
import SpaceSourceReader from "../../reading/presentation/SpaceSourceReader.vue";
import {
  clampSourceReaderWidth,
  SOURCE_READER_WIDTH_DEFAULT,
  SOURCE_READER_WIDTH_MAX,
  SOURCE_READER_WIDTH_MIN,
  type SourceReadingTarget,
} from "../../reading/source-reader-target";

const props = defineProps<{
  active: boolean;
  target?: SourceReadingTarget | undefined;
}>();
const emit = defineEmits<{ close: [] }>();
const sourceVisible = computed(() => props.active && Boolean(props.target));
const panelWidth = shallowRef(SOURCE_READER_WIDTH_DEFAULT);

function updatePanelWidth(size: number) {
  const clamped = clampSourceReaderWidth(size);
  if (clamped !== undefined) panelWidth.value = clamped;
}
</script>

<template>
  <SplitterGroup direction="horizontal" class="min-h-0 flex-1 outline-none">
    <SplitterPanel id="document-editor" :order="1" :min-size="32" class="relative flex min-h-0 flex-1">
      <slot />
    </SplitterPanel>
    <template v-if="sourceVisible">
      <UiSplitterHandle />
      <SplitterPanel
        id="document-source-reader"
        :order="2"
        :min-size="SOURCE_READER_WIDTH_MIN"
        :default-size="panelWidth"
        :max-size="SOURCE_READER_WIDTH_MAX"
        class="flex h-full min-h-0 flex-col border-l border-(--knovum-window-border-color) bg-surface-panel"
        @resize="updatePanelWidth"
      >
        <SpaceSourceReader host="document" :target="target" @close="emit('close')" />
      </SplitterPanel>
    </template>
  </SplitterGroup>
</template>
