<script setup lang="ts">
import { RefreshCw } from "@lucide/vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";

defineProps<{ disabled: boolean; checking: boolean }>();
const emit = defineEmits<{ check: [] }>();
</script>

<template>
  <UiButton
    :disabled="disabled"
    data-testid="check-mcp-connection"
    size="sm"
    variant="secondary"
    @click="emit('check')"
  >
    <UiSpinner v-if="checking" class="size-3.5" />
    <RefreshCw v-else class="size-3.5" />
    <span>{{ $t("extensions.connections.check") }}</span>
  </UiButton>
</template>
