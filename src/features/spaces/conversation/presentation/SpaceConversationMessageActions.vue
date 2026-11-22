<script setup lang="ts">
import { GitBranch, RefreshCw, SquarePlus } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiCopyButton from "../../../../shared/ui/UiCopyButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

/** What can be done with a finished answer: keep it, branch from it or ask for it again. */
defineProps<{ text: string; busy: boolean; canSave: boolean }>();
const emit = defineEmits<{ save: []; branch: []; regenerate: [] }>();
const { t } = useI18n();
</script>

<template>
  <div
    class="message-actions -ml-1 flex items-center gap-0.5 opacity-0 transition-opacity duration-120 group-focus-within:opacity-100 group-hover:opacity-100"
  >
    <UiCopyButton
      :text="text"
      :label="t('spaces.conversation.messages.copy')"
      :copied-label="t('spaces.conversation.messages.copied')"
      :error-message="t('spaces.conversation.messages.copyFailed')"
    />
    <UiIconButton
      :label="t('spaces.conversation.capture.saveAnswer')"
      tooltip
      size="sm"
      :disabled="!canSave"
      @click="emit('save')"
    >
      <SquarePlus class="size-3.5" />
    </UiIconButton>
    <UiIconButton
      :label="t('spaces.conversation.conversations.branch')"
      tooltip
      size="sm"
      :disabled="busy"
      @click="emit('branch')"
    >
      <GitBranch class="size-3.5" />
    </UiIconButton>
    <UiIconButton
      :label="t('spaces.conversation.regenerate')"
      tooltip
      size="sm"
      :disabled="busy"
      @click="emit('regenerate')"
    >
      <RefreshCw class="size-3.5" />
    </UiIconButton>
  </div>
</template>

<style scoped>
/*
 * The row sits below text with fractional line heights. WebKit composites it only while the fade
 * runs and snaps that layer to whole pixels, so the buttons jumped on every hover in and out.
 * Keeping the layer for good stops the switch.
 */
.message-actions {
  will-change: opacity;
}

@media (hover: none) {
  .message-actions {
    opacity: 1;
  }
}
</style>
