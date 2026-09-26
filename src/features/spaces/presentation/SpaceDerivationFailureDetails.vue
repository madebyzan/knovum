<script setup lang="ts">
import { RefreshCw, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import type { DerivationFailure } from "../knowledge-derivations/failure-metadata";

defineProps<{
  failure: DerivationFailure;
  canRetry?: boolean;
  retrying?: boolean;
}>();
const emit = defineEmits<{ remove: []; retry: [] }>();
const { t } = useI18n();
</script>

<template>
  <div class="space-y-3">
    <p class="text-caption leading-relaxed break-words text-danger">{{ failure.error }}</p>
    <p class="text-micro text-foreground-tertiary">{{ new Date(failure.completedAt).toLocaleString() }}</p>
    <div class="flex items-center gap-2">
      <UiButton v-if="canRetry" size="sm" variant="secondary" :disabled="retrying" @click="emit('retry')">
        <RefreshCw class="size-(--knovum-icon-size-xs)" :class="{ 'animate-spin': retrying }" />{{
          t("common.retry")
        }}
      </UiButton>
      <UiButton size="sm" variant="danger" :disabled="retrying" @click="emit('remove')">
        <Trash2 class="size-(--knovum-icon-size-xs)" />{{ t("spaces.creation.shared.removeFailedOutput") }}
      </UiButton>
    </div>
  </div>
</template>
