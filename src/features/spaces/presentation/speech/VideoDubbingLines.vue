<script setup lang="ts">
import { useVirtualizer } from "@tanstack/vue-virtual";
import { computed, useTemplateRef, type ComponentPublicInstance } from "vue";

import type { VideoLineDraft } from "../../speech/video-dubbing-draft";
import VideoDubbingLineEditor from "./VideoDubbingLineEditor.vue";

const props = defineProps<{
  lines: readonly VideoLineDraft[];
  voiceOptions: readonly { value: string; label: string }[];
  deliveryInstructionVoiceIds: ReadonlySet<string>;
  lexiconValid: boolean;
  showSource: boolean;
  disabled: boolean;
}>();
const emit = defineEmits<{
  update: [index: number, line: VideoLineDraft];
  remove: [index: number];
  preview: [text: string, revisionId: string, deliveryInstruction?: string];
  source: [startMs: number, endMs: number];
}>();
const scroller = useTemplateRef<HTMLElement>("scroller");
const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.lines.length,
    getScrollElement: () => scroller.value,
    getItemKey: (index: number) => props.lines[index]!.key,
    estimateSize: () => 260,
    overscan: 4,
  })),
);
const rows = computed(() => virtualizer.value.getVirtualItems());
const total = computed(() => virtualizer.value.getTotalSize());
function measure(element: Element | ComponentPublicInstance | null) {
  const row = element instanceof Element ? element : element?.$el;
  if (row instanceof Element) virtualizer.value.measureElement(row);
}
</script>

<template>
  <div ref="scroller" class="max-h-96 overflow-y-auto">
    <div class="relative" :style="{ height: `${total}px` }">
      <div
        v-for="row in rows"
        :key="String(row.key)"
        :ref="measure"
        :data-index="row.index"
        class="absolute top-0 left-0 w-full pb-2"
        :style="{ transform: `translateY(${row.start}px)` }"
      >
        <VideoDubbingLineEditor
          :model-value="lines[row.index]!"
          :voice-options
          :delivery-instruction-voice-ids
          :lexicon-valid
          :show-source
          :disabled
          @update:model-value="emit('update', row.index, $event)"
          @remove="emit('remove', row.index)"
          @preview="(text, revision, instruction) => emit('preview', text, revision, instruction)"
          @source="(start, end) => emit('source', start, end)"
        />
      </div>
    </div>
  </div>
</template>
