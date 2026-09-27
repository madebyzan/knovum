<script setup lang="ts">
import { Pencil } from "@lucide/vue";
import { computed, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { formatDurationMs } from "../../media/format-duration";
import type { SpaceTranscriptEdit, SpaceTranscriptSegment } from "../../media/knowledge-schema";
import { useNodeDraft } from "../use-node-draft";

const props = defineProps<{
  nodeId: string;
  segment: SpaceTranscriptSegment;
  index: number;
  active?: boolean;
  disabled?: boolean;
}>();
const emit = defineEmits<{
  seek: [positionMs: number];
  save: [payload: SpaceTranscriptEdit];
  editing: [active: boolean];
}>();
const { t } = useI18n();

const dialogOpen = ref(false);
const nodeDraft = useNodeDraft(
  computed(() => props.nodeId),
  "transcript",
);
watch(dialogOpen, (open) => {
  if (!open) {
    nodeDraft.end();
    emit("editing", false);
  }
});
watch(
  () => props.nodeId,
  () => (dialogOpen.value = false),
);
const draft = reactive({ text: props.segment.text });

watch(
  () => props.segment.text,
  () => {
    if (!dialogOpen.value) draft.text = props.segment.text;
  },
);

function openEdit() {
  if (!nodeDraft.begin()) return;
  draft.text = props.segment.text;
  dialogOpen.value = true;
  emit("editing", true);
}

function cancel() {
  dialogOpen.value = false;
  nodeDraft.end();
}

function save() {
  const cleanText = draft.text.trim();
  if (!cleanText) return;
  nodeDraft.end();
  emit("save", { index: props.index, text: cleanText });
  dialogOpen.value = false;
}
</script>

<template>
  <div
    class="group border-b border-divider/50 transition-colors duration-100"
    :class="active ? 'bg-(--knovum-sidebar-item-background-selected)' : 'hover:bg-control-hover/60'"
  >
    <div class="flex gap-2 px-2.5 py-1.5">
      <button
        type="button"
        class="w-11 shrink-0 pt-0.5 text-left font-mono text-micro font-medium text-foreground-tertiary tabular-nums hover:text-foreground"
        @click="emit('seek', segment.startMs)"
      >
        {{ formatDurationMs(segment.startMs) }}
      </button>

      <div
        class="min-w-0 flex-1"
        :class="active ? 'border-l-2 border-accent pl-2' : 'border-l-2 border-transparent pl-2'"
      >
        <p
          class="text-caption leading-snug break-words whitespace-pre-wrap"
          :class="active ? 'text-foreground' : 'text-foreground-secondary'"
        >
          {{ segment.text }}
        </p>
      </div>

      <UiIconButton
        class="shrink-0 transition-opacity"
        :class="active ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
        :label="t('common.edit')"
        size="sm"
        :disabled="disabled || !nodeDraft.available.value"
        @click="openEdit"
      >
        <Pencil class="size-(--knovum-icon-size-xs) text-foreground-tertiary hover:text-foreground" />
      </UiIconButton>
    </div>

    <UiDialog v-model:open="dialogOpen" :title="t('spaces.transcript.text.editSegment')">
      <div class="space-y-4 py-1">
        <label class="block text-micro font-medium text-foreground-tertiary">
          {{ t("spaces.transcript.text.title") }}
          <UiTextarea v-model="draft.text" class="mt-1 w-full resize-none" :rows="4" />
        </label>
      </div>
      <template #footer>
        <UiButton size="sm" variant="ghost" @click="cancel">{{ t("common.cancel") }}</UiButton>
        <UiButton size="sm" variant="primary" :disabled="!draft.text.trim()" @click="save">
          {{ t("common.save") }}
        </UiButton>
      </template>
    </UiDialog>
  </div>
</template>
