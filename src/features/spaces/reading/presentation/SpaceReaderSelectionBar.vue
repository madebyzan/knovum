<script setup lang="ts">
import { Copy, FilePlus2, ListPlus, MessageSquareQuote, StickyNote } from "@lucide/vue";
import { useElementSize } from "@vueuse/core";
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { useNodeActionBarDensity } from "../../canvas/presentation/selection/use-node-action-bar-density";
import type { AppendNoteTarget } from "./use-reader-capture";
import type { SourceReaderSelectionAnchor } from "./use-reader-selection";

const EDGE_MARGIN = 8;

const props = defineProps<{
  anchor: SourceReaderSelectionAnchor;
  containerWidth: number;
  canSave: boolean;
  selectedAppendTarget?: AppendNoteTarget | undefined;
  existingNoteTargets: readonly AppendNoteTarget[];
}>();

const emit = defineEmits<{
  createNote: [];
  createEvidence: [];
  append: [targetId?: string];
  ask: [];
  copy: [];
}>();

const { locale, t } = useI18n();
const bar = useTemplateRef<HTMLElement>("bar");
const pickingNote = shallowRef(false);
const hasAppend = computed(() => Boolean(props.selectedAppendTarget || props.existingNoteTargets.length));
const availableWidth = computed(() => Math.max(0, props.containerWidth - EDGE_MARGIN * 2));
const contentIdentity = computed(() => `${locale.value}:${hasAppend.value}`);
const { compact, ready } = useNodeActionBarDensity(bar, availableWidth, contentIdentity);
const { width: barWidth } = useElementSize(bar);

const clampedLeft = computed(() => {
  if (props.containerWidth <= 0 || barWidth.value <= 0) return props.anchor.left;
  const half = barWidth.value / 2;
  const minLeft = half + EDGE_MARGIN;
  const maxLeft = Math.max(minLeft, props.containerWidth - half - EDGE_MARGIN);
  return Math.min(Math.max(props.anchor.left, minLeft), maxLeft);
});

const barStyle = computed(() => ({
  left: `${clampedLeft.value}px`,
  top: `${props.anchor.top}px`,
  maxWidth: availableWidth.value > 0 ? `${availableWidth.value}px` : undefined,
}));

watch(
  () => props.selectedAppendTarget,
  () => {
    pickingNote.value = false;
  },
);

function handleAppendTrigger() {
  if (props.selectedAppendTarget) {
    emit("append", props.selectedAppendTarget.id);
    return;
  }
  pickingNote.value = !pickingNote.value;
}

function chooseNote(targetId: string) {
  pickingNote.value = false;
  emit("append", targetId);
}
</script>

<template>
  <div
    ref="bar"
    class="absolute z-30 -translate-x-1/2"
    :class="[
      anchor.placement === 'top' && '-translate-y-full',
      availableWidth > 0 && !ready ? 'invisible' : 'visible',
    ]"
    :style="barStyle"
    @mousedown.prevent
  >
    <div
      role="toolbar"
      :aria-label="t('spaces.conversation.capture.selection')"
      class="flex flex-col rounded-panel border border-stroke-strong bg-surface-overlay p-1 shadow-popover backdrop-blur-md select-none"
    >
      <div class="flex items-center gap-0.5 whitespace-nowrap">
        <UiTooltip :text="t('spaces.conversation.capture.saveSelection')" :disabled="!compact" side="top">
          <button
            type="button"
            :aria-label="t('spaces.conversation.capture.saveSelection')"
            :disabled="!canSave"
            class="inline-flex h-7 shrink-0 items-center justify-center rounded-md text-caption font-medium text-fg-primary transition-colors hover:bg-surface-control-hover active:bg-surface-control-pressed disabled:opacity-40"
            :class="compact ? 'w-7 px-0' : 'gap-1.5 px-2.5'"
            @click="emit('createNote')"
          >
            <StickyNote class="size-3.5 shrink-0 text-fg-secondary" :stroke-width="1.65" />
            <span v-if="!compact" class="whitespace-nowrap">{{
              t("spaces.conversation.capture.saveSelection")
            }}</span>
          </button>
        </UiTooltip>
        <UiTooltip :text="t('spaces.conversation.evidence.extract')" :disabled="!compact" side="top">
          <button
            type="button"
            :aria-label="t('spaces.conversation.evidence.extract')"
            :disabled="!canSave"
            class="inline-flex h-7 shrink-0 items-center justify-center rounded-md text-caption font-medium text-fg-primary transition-colors hover:bg-surface-control-hover active:bg-surface-control-pressed disabled:opacity-40"
            :class="compact ? 'w-7 px-0' : 'gap-1.5 px-2.5'"
            @click="emit('createEvidence')"
          >
            <FilePlus2 class="size-3.5 shrink-0 text-fg-secondary" :stroke-width="1.65" />
            <span v-if="!compact" class="whitespace-nowrap">{{
              t("spaces.conversation.evidence.extract")
            }}</span>
          </button>
        </UiTooltip>
        <span class="mx-1 h-4 w-px shrink-0 bg-stroke-divider" aria-hidden="true" />
        <UiIconButton
          v-if="hasAppend"
          :label="
            selectedAppendTarget
              ? t('spaces.conversation.capture.appendTo', { title: selectedAppendTarget.title })
              : t('spaces.conversation.capture.appendTo', { title: '…' })
          "
          tooltip
          size="sm"
          @click="handleAppendTrigger"
        >
          <ListPlus class="size-3.5" />
        </UiIconButton>
        <UiIconButton :label="t('spaces.conversation.capture.followUp')" tooltip size="sm" @click="emit('ask')">
          <MessageSquareQuote class="size-3.5" />
        </UiIconButton>
        <UiIconButton :label="t('spaces.conversation.messages.copy')" tooltip size="sm" @click="emit('copy')">
          <Copy class="size-3.5" />
        </UiIconButton>
      </div>
      <div
        v-if="pickingNote && !selectedAppendTarget && existingNoteTargets.length"
        class="mt-1 max-h-44 min-w-48 overflow-y-auto border-t border-stroke-divider pt-1"
      >
        <button
          v-for="target in existingNoteTargets"
          :key="target.id"
          type="button"
          class="flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-caption text-fg-primary hover:bg-surface-control-hover"
          @click="chooseNote(target.id)"
        >
          <StickyNote class="size-3.5 shrink-0 text-fg-tertiary" />
          <span class="truncate">{{ target.title }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
