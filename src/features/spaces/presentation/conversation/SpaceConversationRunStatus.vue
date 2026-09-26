<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { WaitReason } from "../../contracts/generated/WaitReason";

defineProps<{ state?: string | undefined; waitReason: WaitReason | null; hasPendingDecision: boolean }>();
const emit = defineEmits<{ resume: [] }>();
const { t } = useI18n();
</script>

<template>
  <div v-if="state === 'paused'" class="flex shrink-0 items-center gap-2 px-3 py-2 text-caption text-warning">
    <span class="min-w-0 flex-1">{{ t("spaces.assistant.tasks.paused") }}</span>
    <UiButton size="sm" variant="secondary" @click="emit('resume')">{{
      t("spaces.assistant.tasks.resume")
    }}</UiButton>
  </div>
  <p
    v-else-if="state === 'waiting' && !hasPendingDecision"
    class="shrink-0 px-3 py-2 text-caption text-foreground-secondary"
  >
    {{
      t(
        waitReason === "approval"
          ? "spaces.assistant.tasks.needsAttention"
          : "spaces.assistant.tasks.waitingForResult",
      )
    }}
  </p>
</template>
