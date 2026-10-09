<script setup lang="ts">
import { Pencil } from "@lucide/vue";
import { computed, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { formatDurationMs } from "../../media/format-duration";
import type { SpaceMediaChapter } from "../../media/schema/knowledge";
import { useNodeDraft } from "../inspector/use-node-draft";

const props = defineProps<{
  nodeId: string;
  chapter: SpaceMediaChapter;
  index: number;
  active?: boolean;
  disabled?: boolean;
}>();
const emit = defineEmits<{
  seek: [positionMs: number];
  save: [index: number, title: string, summary: string];
  shift: [index: number, direction: -1 | 1];
}>();
const { t } = useI18n();

const dialogOpen = ref(false);
const nodeDraft = useNodeDraft(computed(() => props.nodeId));
watch(dialogOpen, (open) => {
  if (!open) nodeDraft.end();
});
watch(
  () => props.nodeId,
  () => (dialogOpen.value = false),
);
const draft = reactive({ title: props.chapter.title, summary: props.chapter.summary });

function openEdit() {
  if (!nodeDraft.begin()) return;
  draft.title = props.chapter.title;
  draft.summary = props.chapter.summary;
  dialogOpen.value = true;
}

function cancel() {
  dialogOpen.value = false;
  nodeDraft.end();
}

function save() {
  const cleanTitle = draft.title.trim();
  if (!cleanTitle) return;
  nodeDraft.end();
  emit("save", props.index, cleanTitle, draft.summary.trim());
  dialogOpen.value = false;
}
</script>

<template>
  <article
    class="group flex items-start gap-2 rounded-md px-2 py-1.5 transition-colors duration-100 hover:bg-surface-control-hover"
    :class="
      active ? 'bg-(--knovum-sidebar-item-bg-selected) font-medium text-fg-primary' : 'text-fg-secondary'
    "
  >
    <button
      type="button"
      class="w-10 shrink-0 pt-0.5 text-left font-mono text-caption-sm font-medium text-fg-tertiary tabular-nums hover:text-fg-primary"
      :title="t('spaces.transcript.chapters.seek')"
      @click="emit('seek', chapter.startMs)"
    >
      {{ formatDurationMs(chapter.startMs) }}
    </button>

    <div class="min-w-0 flex-1 space-y-0.5">
      <strong class="block text-caption leading-snug font-medium break-words text-fg-primary">
        {{ chapter.title }}
      </strong>
      <p
        v-if="chapter.summary"
        class="text-caption-sm leading-relaxed break-words whitespace-pre-wrap text-fg-secondary"
      >
        {{ chapter.summary }}
      </p>
    </div>

    <UiIconButton
      :label="t('common.edit')"
      size="sm"
      :disabled="disabled || !nodeDraft.available.value"
      class="shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
      @click="openEdit"
    >
      <Pencil class="size-3 text-fg-tertiary hover:text-fg-primary" />
    </UiIconButton>

    <UiDialog v-model:open="dialogOpen" :title="t('spaces.transcript.chapters.edit')">
      <div class="space-y-4 py-1">
        <label class="block text-caption-sm font-medium text-fg-tertiary">
          {{ t("spaces.transcript.chapters.titleField") }}
          <UiInput v-model="draft.title" class="mt-1 w-full" />
        </label>
        <label class="block text-caption-sm font-medium text-fg-tertiary">
          {{ t("spaces.transcript.chapters.summary") }}
          <UiTextarea v-model="draft.summary" class="mt-1 w-full resize-none" :rows="4" />
        </label>
      </div>
      <template #footer>
        <UiButton size="sm" variant="ghost" @click="cancel">{{ t("common.cancel") }}</UiButton>
        <UiButton size="sm" variant="primary" :disabled="!draft.title.trim()" @click="save">
          {{ t("common.save") }}
        </UiButton>
      </template>
    </UiDialog>
  </article>
</template>
