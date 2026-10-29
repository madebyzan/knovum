<script setup lang="ts">
import { Copy, MessageSquareQuote, StickyNote } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

/** Actions on a passage selected in an answer. It never takes focus, so the selection survives the click. */
defineProps<{ canSave: boolean }>();
const emit = defineEmits<{ save: []; followUp: []; copy: [] }>();
const { t } = useI18n();
</script>

<template>
  <div
    role="toolbar"
    :aria-label="t('spaces.conversation.capture.selection')"
    class="flex items-center gap-0.5 rounded-md border border-stroke-default bg-surface-raised p-0.5 shadow-control"
    @mousedown.prevent
  >
    <UiIconButton
      :label="t('spaces.conversation.capture.saveSelection')"
      tooltip
      size="sm"
      :disabled="!canSave"
      @click="emit('save')"
    >
      <StickyNote class="size-3.5" />
    </UiIconButton>
    <UiIconButton
      :label="t('spaces.conversation.capture.followUp')"
      tooltip
      size="sm"
      @click="emit('followUp')"
    >
      <MessageSquareQuote class="size-3.5" />
    </UiIconButton>
    <UiIconButton :label="t('spaces.conversation.messages.copy')" tooltip size="sm" @click="emit('copy')">
      <Copy class="size-3.5" />
    </UiIconButton>
  </div>
</template>
