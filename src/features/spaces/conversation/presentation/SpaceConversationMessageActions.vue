<script setup lang="ts">
import { Check, Copy, GitBranch, SquarePlus, RefreshCw } from "@lucide/vue";
import { useTimeoutFn } from "@vueuse/core";
import { writeText } from "tauri-plugin-clipboard-api";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";

/** What can be done with a finished answer: keep it, branch from it or ask for it again. */
const props = defineProps<{ text: string; busy: boolean; canSave: boolean }>();
const emit = defineEmits<{ save: []; branch: []; regenerate: [] }>();
const { t } = useI18n();
const copyFailed = shallowRef(false);
const copied = shallowRef(false);
const { start: resetCopied } = useTimeoutFn(() => (copied.value = false), 1600, { immediate: false });

async function copy() {
  try {
    await writeText(props.text);
    copyFailed.value = false;
    copied.value = true;
    resetCopied();
  } catch {
    copyFailed.value = true;
    copied.value = false;
  }
}
</script>

<template>
  <p v-if="copyFailed" role="alert" class="text-caption text-fg-danger">
    {{ t("spaces.conversation.messages.copyFailed") }}
  </p>
  <div
    class="message-actions flex items-center gap-0.5 opacity-0 transition-opacity duration-100 group-focus-within:opacity-100 group-hover:opacity-100"
  >
    <UiIconButton
      :label="t(copied ? 'spaces.conversation.messages.copied' : 'spaces.conversation.messages.copy')"
      size="sm"
      @click="copy"
    >
      <Check v-if="copied" class="size-3.5 text-fg-accent" />
      <Copy v-else class="size-3.5" />
    </UiIconButton>
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
