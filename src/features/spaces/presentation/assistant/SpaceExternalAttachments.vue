<script setup lang="ts">
import { Paperclip, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { AgentAttachment } from "../../agent-creation/use-agent-attachments";

defineProps<{ attachments: readonly AgentAttachment[]; busy: boolean }>();
const emit = defineEmits<{ choose: []; remove: [path: string] }>();
const { t } = useI18n();
</script>

<template>
  <div class="flex flex-wrap items-center gap-1.5">
    <UiButton type="button" size="sm" variant="secondary" :disabled="busy" @click="emit('choose')">
      <Paperclip class="size-(--knovum-icon-size-xs)" />
      {{ t("spaces.assistant.addExternalAttachment") }}
    </UiButton>
    <span
      v-for="attachment in attachments"
      :key="attachment.path"
      class="flex max-w-full items-center gap-1 rounded-control bg-control px-2 py-1 text-caption"
    >
      <span class="max-w-36 truncate">{{ attachment.name }}</span>
      <UiButton
        type="button"
        size="sm"
        variant="ghost"
        class="size-4 px-0 shadow-none"
        :aria-label="t('spaces.assistant.removeExternalAttachment', { name: attachment.name })"
        :disabled="busy"
        @click="emit('remove', attachment.path)"
      >
        <X class="size-(--knovum-icon-size-xs)" />
      </UiButton>
    </span>
  </div>
</template>
