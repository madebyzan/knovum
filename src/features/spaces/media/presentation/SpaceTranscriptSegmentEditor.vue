<script setup lang="ts">
import { Pencil } from "@lucide/vue";
import { computed, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { useNodeDraft } from "../../inspection/presentation/use-node-draft";
import { formatDurationMs } from "../format-duration";
import type { SpaceTranscriptEdit } from "../schema/knowledge";
import type { SpaceTranscriptSegment } from "../schema/transcript";

const props = defineProps<{
  nodeId: string;
  segment: SpaceTranscriptSegment;
  index: number;
  active?: boolean;
  disabled?: boolean;
  selected?: boolean;
  selectionActive?: boolean;
  selectionDisabled?: boolean;
}>();
const emit = defineEmits<{
  seek: [positionMs: number];
  save: [payload: SpaceTranscriptEdit];
  editing: [active: boolean];
  select: [selected: boolean];
}>();
const { t } = useI18n();

const dialogOpen = ref(false);
const nodeDraft = useNodeDraft(computed(() => props.nodeId));
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
    class="group relative rounded-md transition-colors duration-100"
    :class="active ? 'bg-(--knovum-sidebar-item-bg-selected)' : 'hover:bg-surface-control-hover/70'"
  >
    <div
      v-if="active"
      class="absolute top-1.5 bottom-1.5 left-0 w-0.5 rounded-r bg-surface-accent"
      aria-hidden="true"
    />
    <div class="flex items-start gap-2 px-1.5 py-1.5">
      <button
        type="button"
        class="flex h-5 w-10 shrink-0 items-center justify-start rounded-xs font-mono text-caption-sm font-medium tabular-nums transition-colors hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :class="active ? 'font-semibold text-fg-accent' : 'text-fg-tertiary'"
        :title="t('spaces.transcript.chapters.seek')"
        @click="emit('seek', segment.startMs)"
      >
        {{ formatDurationMs(segment.startMs) }}
      </button>

      <p
        class="min-w-0 flex-1 text-caption leading-5 wrap-break-word whitespace-pre-wrap select-text"
        :class="active ? 'font-medium text-fg-primary' : 'text-fg-secondary'"
      >
        {{ segment.text }}
      </p>

      <div
        class="flex h-5 shrink-0 items-center gap-1 transition-opacity duration-100"
        :class="
          selected || selectionActive
            ? 'opacity-100'
            : 'opacity-0 group-focus-within:opacity-100 group-hover:opacity-100'
        "
      >
        <UiIconButton
          class="size-5"
          :label="t('common.edit')"
          tooltip
          size="sm"
          :disabled="disabled || !nodeDraft.available.value"
          @click="openEdit"
        >
          <Pencil class="size-3" />
        </UiIconButton>
        <UiCheckbox
          :model-value="Boolean(selected)"
          :disabled="selectionDisabled"
          :label="t('spaces.transcript.text.selectForAlignment')"
          @update:model-value="emit('select', $event)"
        />
      </div>
    </div>

    <UiDialog v-model:open="dialogOpen" :title="t('spaces.transcript.text.editSegment')">
      <div class="space-y-4 py-1">
        <label class="block text-caption-sm font-medium text-fg-tertiary">
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
